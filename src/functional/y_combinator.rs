use std::marker::PhantomData;

// Implements Y-combinator without using things like dyn or Rc<RefCell<>>
// Rust allows direct recursion so Y-combinator is not needed. Implement just for learning.

// In lambda calculus, there are only values and lambda expressions,
// so a recursive function cannot be directly written,
// such as factorial = n -> if (n = 0) then 1 else n * factorial(n - 1)
// one way to fix is to pass the function itself as an argument,
// self_accepting_factorial = f -> n -> if (n = 0) then 1 else n * f(f)(n - 1)
// factorial = self_accepting_factorial(self_accepting_factorial)
// another way is to write a function with fixed point that takes its "full version" as argument
// fixed_point_factorial = f -> n -> if (n = 0) then 1 else n * f(n - 1)
// It has a fixed point: fixed_point_factorial factorial = factorial, and the fixed point is the result recursive function.
// Y combinator can obtain the fixed point of a function. Y(fixed_point_factorial) = factorial
// f is the function having fixed point r, f(r) = r
// m is the function in self-accepting form, r = m(m)
// f(r) = f(m(m)) = "the output of the recursive function"
// The self-accepting function m can "come out of nowhere" because its argument is itself.
// m = (m -> f(r)) = ( m -> f(m(m)) )
// Then, we can get Y combinator:
// Y(f) = r = m(m) = (m -> f(m(m))) (m -> f(m(m)))
// Y = f -> (m -> f(m(m))) (m -> f(m(m)))
// let's write down the type of them:
// FuncHavingFixedPoint<Input, Output> = (Input -> Output) -> Input -> Output
// SelfAcceptingFunc<Input, Output> = SelfAcceptingFunc<Input, Output> -> Input -> Output
// f: FuncHavingFixedPoint<Input, Output>, m: SelfAcceptingFunc<Input, Output>
// m(m): Input -> Output, f(m(m)): Input -> Output,
// ( m -> f(m(m)) ): SelfAcceptingFunc<Input, Output> (it has the same type as m)
// ( (m -> f(m(m))) (m -> f(m(m))) ): Input -> Output

// Once having a self-accepting function, applying it to itself gives the result.

// It represents a recursive function that takes itself as an argument.
// for a curried fixed point func f, f g = g
pub trait FuncHavingFixedPoint<Input, Output> {
    // in non-curry form
    fn eval<FuncArg>(&self, func_arg: &FuncArg, input: Input) -> Output
        where FuncArg: Fn(Input) -> Output;
}

// Note: self-accepting func is different to fixed point func.
// to call a self-accepting func f, you need f(f, input), where the first argument's type is the same as f.
// to call a fixed point func f, you need f(f, input),
// where the first argument self_func can be called without passing itself, whose type is different to f.
trait SelfAcceptingFunc<Input, Output> {
    // in non-curry form
    fn eval<SelfFunc>(&self, self_func: &SelfFunc, input: Input) -> Output
        where SelfFunc: SelfAcceptingFunc<Input, Output>;
}

// apply a self-accepting func to itself to get the result.
// m(m)
// un curried: input -> m(m, input)
fn self_accepting_func_apply_itself<Input, Output, SelfAcceptingFuncImpl: Copy>(
    self_accepting_func: SelfAcceptingFuncImpl
) -> impl Fn(Input) -> Output
    where SelfAcceptingFuncImpl: SelfAcceptingFunc<Input, Output>
{
    return move |input| {
        self_accepting_func.eval(&self_accepting_func, input)
    };
}

struct FixedPointFuncWrappedAsSelfAcceptingFunc<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{
    fixed_point_func: &'a FixedPointFuncImpl,
    __phantom: PhantomData<(Input, Output)>,
}

// impl Clone for it
impl<'a, Input, Output, FixedPointFuncImpl> Clone for FixedPointFuncWrappedAsSelfAcceptingFunc<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{
    fn clone(&self) -> Self {
        FixedPointFuncWrappedAsSelfAcceptingFunc {
            fixed_point_func: self.fixed_point_func,
            __phantom: PhantomData,
        }
    }
}

// impl Copy for it
impl<'a, Input, Output, FixedPointFuncImpl> Copy for FixedPointFuncWrappedAsSelfAcceptingFunc<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{}

impl<'a, Input, Output, FixedPointFuncImpl> SelfAcceptingFunc<Input, Output>
for FixedPointFuncWrappedAsSelfAcceptingFunc<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{
    fn eval<SelfFunc>(&self, self_func: &SelfFunc, input: Input) -> Output
        where SelfFunc: SelfAcceptingFunc<Input, Output>
    {
        // given f, the function having fixed point,
        // m -> f(m(m)) is a self-accepting function
        // applying de-currying rule: a -> b => (a, input) -> b(input), a(b) => a(b, input)
        // it becomes (m, input) -> f(input2 -> m(m, input2), input)
        self.fixed_point_func.eval(&|input2| { self_func.eval(self_func, input2) }, input)
    }
}

// f(r) = r, Y(f) = r = m(m), m = ( m -> f(m(m)) )
// where f is a function having fixed point r,
// m is a self-accepting func curried.
pub fn y_combinator<'a, Input: 'a, Output: 'a, FixedPointFuncImpl>(
    fixed_point_func: &'a FixedPointFuncImpl
) -> impl Fn(Input) -> Output + 'a
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{
    self_accepting_func_apply_itself(
        FixedPointFuncWrappedAsSelfAcceptingFunc {
            fixed_point_func,
            __phantom: PhantomData,
        }
    )
}

// When self-reference is allowed, Y combinator is not needed.
pub struct SelfReferencialFixedPointApplier<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{
    fixed_point_func: &'a FixedPointFuncImpl,
    __phantom: PhantomData<(Input, Output)>,
}

impl<'a, Input, Output, FixedPointFuncImpl> SelfReferencialFixedPointApplier<'a, Input, Output, FixedPointFuncImpl>
    where FixedPointFuncImpl: FuncHavingFixedPoint<Input, Output>
{
    pub fn new(fixed_point_func: &'a FixedPointFuncImpl) -> Self {
        SelfReferencialFixedPointApplier { fixed_point_func, __phantom: PhantomData }
    }
    
    pub fn eval(&self, input: Input) -> Output {
        self.fixed_point_func.eval(&|input| self.eval(input), input)
    }
}

struct FactorialFunc {}

impl FuncHavingFixedPoint<u32, u32> for FactorialFunc {
    fn eval<FuncArg>(&self, func_arg: &FuncArg, input: u32) -> u32
        where FuncArg: Fn(u32) -> u32
    {
        if input == 0 {
            1
        } else {
            input * func_arg(input - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_factorial() {
        let factorial_func = FactorialFunc {};
        let factorial = y_combinator(&factorial_func);
        assert_eq!(factorial(5), 120);
        
        let factorial = SelfReferencialFixedPointApplier::new(&factorial_func);
        assert_eq!(factorial.eval(5), 120);
    }
}