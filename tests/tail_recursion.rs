use kisp::assert_match;
use kisp::testutils::quick_result;
use kisp::value::{EvalResult, EvalValue};
use kisp::value::numeric::Numeric;

#[test]
fn sum_of_n(){
    let (value, _) = quick_result(
        "
        (fn sum [n]
            [
                (fn iter [n acc]
                    (if (>= 0 n)
                        acc
                        (iter (- n 1) (+ acc n))
                    )
                )
                (iter n 0)
            ]
        )
        (sum 100)
        "
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==5050);
}
#[test]
fn tail_call_inside_block(){
    let (value, _) = quick_result(
        "
        (fn id [x] x)
        (fn loop [n]
            (if (>= 0 n)
                0
                [(id n) (loop (- n 1))]))
        (loop 20000)
        "
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==0);
}

#[test]
fn tail_call_with_accumulator_deep(){
    let (value, _) = quick_result(
        "
        (fn iter [n acc] (if (>= 0 n) acc (iter (- n 1) (+ acc 1))))
        (iter 100000 0)
        "
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==100000);
}
