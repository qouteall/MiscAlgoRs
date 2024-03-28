use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;

// The Value type is Clone,
// because if not, it needs to return reference of value in cache, which indirectly borrows the cache,
// and easily cause borrow checker issues.
// The implementor should use things like arena if value is heavy.
pub trait Cache<Key, Value> {
    fn get_from_cache<>(&self, key: &Key) -> Option<Value>;
    
    fn put_to_cache(&mut self, key: &Key, value: Value);
}

// cannot directly implement IndexMut for Vec<Value> because of coherence rules.
impl<Value: Clone> Cache<usize, Value> for Vec<Option<Value>> {
    fn get_from_cache(&self, key: &usize) -> Option<Value> {
        let index = *key;
        
        if index >= self.len() {
            return None;
        }
        
        self[index].clone()
    }
    
    fn put_to_cache(&mut self, key: &usize, value: Value) {
        let index = *key;
        
        // auto-grow the cache
        if index >= self.len() {
            self.resize(index + 1, None);
        }
        
        self[index] = Some(value);
    }
}

impl<Value: Clone> Cache<usize, Value> for [Option<Value>] {
    fn get_from_cache(&self, key: &usize) -> Option<Value> {
        self[*key].clone()
    }
    
    fn put_to_cache(&mut self, key: &usize, value: Value) {
        self[*key] = Some(value);
    }
}

impl<Key: Eq + Hash + Clone, Value: Clone> Cache<Key, Value> for HashMap<Key, Value> {
    fn get_from_cache(&self, key: &Key) -> Option<Value> {
        self.get(key).cloned()
    }
    
    fn put_to_cache(&mut self, key: &Key, value: Value) {
        self.insert(key.clone(), value);
    }
}

impl<Key: Ord + Clone, Value: Clone> Cache<Key, Value> for BTreeMap<Key, Value> {
    fn get_from_cache(&self, key: &Key) -> Option<Value> {
        self.get(key).cloned()
    }
    
    fn put_to_cache(&mut self, key: &Key, value: Value) {
        self.insert(key.clone(), value);
    }
}

// sometimes we want to use a Vec to be the cache, but Vec cache only support usize key,
// we can use this to map custom type to u32 thus using Vec as cache
struct KeyMappedCacheAccess<
    'a, OriginalKey, MappedKey, Value, CacheImpl, KeyMapFunc
>
    where CacheImpl: Cache<MappedKey, Value>,
          KeyMapFunc: Fn(&OriginalKey) -> MappedKey
{
    cache: &'a mut CacheImpl,
    key_map_func: KeyMapFunc,
    __phantom: PhantomData<(OriginalKey, Value)>,
}

impl<'a, OriginalKey, MappedKey, Value, CacheImpl, KeyMapFunc> Cache<OriginalKey, Value>
for KeyMappedCacheAccess<'a, OriginalKey, MappedKey, Value, CacheImpl, KeyMapFunc>
    where CacheImpl: Cache<MappedKey, Value>,
          KeyMapFunc: Fn(&OriginalKey) -> MappedKey
{
    fn get_from_cache(&self, key: &OriginalKey) -> Option<Value> {
        let mapped_key = (self.key_map_func)(key);
        self.cache.get_from_cache(&mapped_key)
    }
    
    fn put_to_cache(&mut self, key: &OriginalKey, value: Value) {
        let mapped_key = (self.key_map_func)(key);
        self.cache.put_to_cache(&mapped_key, value);
    }
}

// Wrap a function to make it lazy-evaluated.
// (does not work on recursive function)
pub struct LazyEvalNormalFunction<'a, Input, Output, Func, CacheImpl>
    where Func: Fn(&Input) -> Output + 'a, CacheImpl: Cache<Input, Output>
{
    func: &'a Func,
    cache: CacheImpl,
    __phantom: PhantomData<Input>,
}

impl<'a, Input, Output: Clone, Func, CacheImpl> LazyEvalNormalFunction<'a, Input, Output, Func, CacheImpl>
    where Func: Fn(&Input) -> Output,
          CacheImpl: Cache<Input, Output>
{
    pub fn new(func: &'a Func, cache: CacheImpl) -> Self {
        LazyEvalNormalFunction { func, cache, __phantom: PhantomData }
    }
    
    pub fn eval(&mut self, input: &Input) -> Output {
        let cache_query_result = self.cache.get_from_cache(input);
        
        if let Some(value) = cache_query_result {
            return value;
        }
        
        let new_value: Output = (self.func)(input);
        self.cache.put_to_cache(input, new_value.clone());
        new_value
    }
}

impl<'a, Input, Output: Clone, Func, C> FnOnce<(&Input, )> for LazyEvalNormalFunction<'a, Input, Output, Func, C>
    where Func: Fn(&Input) -> Output, C: Cache<Input, Output>
{
    type Output = Output;
    
    extern "rust-call" fn call_once(mut self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

impl<'a, Input, Output: Clone, Func, C> FnMut<(&Input, )> for LazyEvalNormalFunction<'a, Input, Output, Func, C>
    where Func: Fn(&Input) -> Output, C: Cache<Input, Output>
{
    extern "rust-call" fn call_mut(&mut self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

// It represents a recursive function that invokes recursion by invoking the argument.
// Its curried form, denoted f, has a fixed point r, f(r) = r.
// r is the directly-callable recursion function.
// write it in this form to allow lazy evaluation of recursive functions.
pub trait FuncHavingFixedPointMut<Input, Output> {
    // the recursion arg is mutable because it may contain a mutable cache.
    // rust does not yet have mutability polymorphism.
    fn eval<FuncArg>(&self, recursion: &mut FuncArg, input: &Input) -> Output
        where FuncArg: FnMut(&Input) -> Output;
}

// an example of recursive function written in this form.
struct FibonacciFunc {
    invoke_count: Cell<u32>,
}

impl FuncHavingFixedPointMut<usize, usize> for FibonacciFunc {
    fn eval<FuncArg>(&self, recursion: &mut FuncArg, input: &usize) -> usize
        where FuncArg: FnMut(&usize) -> usize
    {
        self.invoke_count.set(self.invoke_count.get() + 1);
        match input {
            0 => 0,
            1 => 1,
            _ => recursion(&(input - 1)) + recursion(&(input - 2)),
        }
    }
}

// directly turn the function with fixed point into normal recursive function,
// without lazy evaluation.
pub struct SimpleFixedPointApplyFunc<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Output>
{
    fixed_point_func: &'a FixedPointFuncImpl,
    __phantom: PhantomData<(Input, Output)>,
}

impl<'a, Input, Output, FixedPointFuncImpl> SimpleFixedPointApplyFunc<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Output>
{
    pub fn new(fixed_point_func: &'a FixedPointFuncImpl) -> Self {
        SimpleFixedPointApplyFunc { fixed_point_func, __phantom: PhantomData }
    }
    
    pub fn eval(&self, input: &Input) -> Output {
        self.fixed_point_func.eval(&mut |input: &Input| self.eval(input), input)
    }
}

impl<'a, Input, Out, FixedPointFuncImpl> FnOnce<(&Input, )> for
SimpleFixedPointApplyFunc<'a, Input, Out, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Out>
{
    type Output = Out;
    
    extern "rust-call" fn call_once(self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

impl<'a, Input, Out, FixedPointFuncImpl> FnMut<(&Input, )> for
SimpleFixedPointApplyFunc<'a, Input, Out, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Out>
{
    extern "rust-call" fn call_mut(&mut self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

impl<'a, Input, Out, FixedPointFuncImpl> Fn<(&Input, )> for
SimpleFixedPointApplyFunc<'a, Input, Out, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Out>
{
    extern "rust-call" fn call(&self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

pub struct LazyEvalFixedPointApplyFunc<'a, Input, Output, CacheImpl, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Output>,
          CacheImpl: Cache<Input, Output>
{
    fixed_point_func: &'a FixedPointFuncImpl,
    cache: CacheImpl,
    __phantom: PhantomData<(Input, Output)>,
}

impl<'a, Input, Output: Clone, CacheImpl, FixedPointFuncImpl> LazyEvalFixedPointApplyFunc<'a, Input, Output, CacheImpl, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Output>,
          CacheImpl: Cache<Input, Output>
{
    pub fn new(fixed_point_func: &'a FixedPointFuncImpl, cache: CacheImpl) -> Self {
        LazyEvalFixedPointApplyFunc { fixed_point_func, cache, __phantom: PhantomData }
    }
    
    pub fn eval(&mut self, input: &Input) -> Output {
        let cache_query_result = self.cache.get_from_cache(&input);
        
        if let Some(value) = cache_query_result {
            return value;
        }
        
        let new_value: Output = self.fixed_point_func.eval(&mut |input2: &Input| self.eval(input2), input);
        self.cache.put_to_cache(&input, new_value.clone());
        new_value
    }
}

impl<'a, Input, Output: Clone, CacheImpl, FixedPointFuncImpl> FnOnce<(&Input, )> for LazyEvalFixedPointApplyFunc<'a, Input, Output, CacheImpl, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Output>,
          CacheImpl: Cache<Input, Output>
{
    type Output = Output;
    
    extern "rust-call" fn call_once(mut self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

impl<'a, Input, Output: Clone, CacheImpl, FixedPointFuncImpl> FnMut<(&Input, )> for LazyEvalFixedPointApplyFunc<'a, Input, Output, CacheImpl, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPointMut<Input, Output>,
          CacheImpl: Cache<Input, Output>
{
    extern "rust-call" fn call_mut(&mut self, args: (&Input, )) -> Self::Output {
        self.eval(args.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lazy_eval_normal_func() {
        let invoke_count: Cell<u32> = Cell::new(0);
        
        let func = |input: &usize| {
            invoke_count.set(invoke_count.get() + 1);
            *input * 2
        };
        
        let cache: Vec<Option<usize>> = Vec::new();
        
        let mut lazy_eval_normal_func = LazyEvalNormalFunction::new(&func, cache);
        
        let result = lazy_eval_normal_func.eval(&10);
        assert_eq!(result, 20);
        
        lazy_eval_normal_func.eval(&10);
        
        let invoke_count = invoke_count.get();
        assert_eq!(invoke_count, 1);
    }
    
    #[test]
    fn test_fibonacci() {
        let fibonacci_func = FibonacciFunc { invoke_count: Cell::new(0) };
        let simple_fixed_point_apply_func =
            SimpleFixedPointApplyFunc::new(&fibonacci_func);
        
        let result = simple_fixed_point_apply_func(&10);
        assert_eq!(result, 55);
        
        let non_cached_invoke_count = fibonacci_func.invoke_count.get();
        
        println!("non-cached invoke count: {}", non_cached_invoke_count);
        
        let mut cache: Vec<Option<usize>> = Vec::new();
        
        let fibonacci_func = FibonacciFunc { invoke_count: Cell::new(0) };
        let mut lazy_eval_fixed_point_apply_func =
            LazyEvalFixedPointApplyFunc::new(&fibonacci_func, cache);
        
        let result = lazy_eval_fixed_point_apply_func(&10);
        assert_eq!(result, 55);
        
        let cached_invoke_count = fibonacci_func.invoke_count.get();
        
        println!("cached invoke count: {}", cached_invoke_count);
        
        assert!(cached_invoke_count < non_cached_invoke_count);
    }
    
    #[test]
    fn test_mapped_cache() {
        // key start from 100000, mapping subtract it by 100000, making Vec-based cache smaller
        let offset = 100000;
        let key_map_func = |key: &usize| key - offset;
        
        let mut cache_vec: Vec<Option<usize>> = Vec::new();
        
        let mut key_mapped_cache = KeyMappedCacheAccess {
            cache: &mut cache_vec,
            key_map_func,
            __phantom: PhantomData,
        };
        
        key_mapped_cache.put_to_cache(&(123 + offset), 456);
        
        assert!(cache_vec.len() < 1000);
        
        assert_eq!(cache_vec[123].unwrap(), 456);
    }
}