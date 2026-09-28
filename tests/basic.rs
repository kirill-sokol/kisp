use kisp::assert_match;

use kisp::testutils::quick_result;
use kisp::value::{EvalResult, EvalValue};
use kisp::value::numeric::Numeric;



#[test]
fn empty_source(){
    let (value, _) = quick_result(
        ""
    ).unwrap();
    assert_match!(value, EvalValue::Unit);
}

#[test]
fn echo_num(){
    let (value, _) = quick_result(
        "1"
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==1);
}

#[test]
fn block(){
    let (value, _) = quick_result(
        "[1 2 3 4 5]"
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==5);
}

#[test]
fn source_block(){
    let (value, _) = quick_result(
        "6 7 8 9 10"
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==10);
}

#[test]
fn unit(){
    let (value, _) = quick_result(
        "()"
    ).unwrap();
    assert_match!(value, EvalValue::Unit);
}

#[test]
fn addition(){
    let (value, _) = quick_result(
        "(+ 1 1)"
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==2);
}

#[test]
fn comments(){
    let (value, _) = quick_result("
        ;hello there
        (+ 1 1)
        ;I should have no effect on the result"
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==2);
}
#[test]
fn closures_capture_defining_scope(){
    let (value, _) = quick_result(
        "
        (fn mk [n] (lambda [y] (+ n y)))
        (let add5 (mk 5))
        (add5 10)
        "
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==15);
}

#[test]
fn lexical_scoping(){
    // f does not see g's argument x, it sees the global x
    let (value, _) = quick_result(
        "
        (let x 1)
        (fn f [] x)
        (fn g [x] (f))
        (g 5)
        "
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==1);
}

#[test]
fn deep_recursion_is_an_error_not_a_crash(){
    // Test threads get 2 MiB of stack; MAX_CALL_DEPTH nested calls in a debug
    // build need more, so run on a thread sized like the main thread (8 MiB).
    let is_err = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| quick_result(
            "
            (fn r [n] (if (= n 0) 0 (+ 1 (r (- n 1)))))
            (r 100000)
            "
        ).is_err())
        .unwrap()
        .join()
        .unwrap();
    assert!(is_err);
}

#[test]
fn float_cast(){
    let (value, _) = quick_result("(float 3)").unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Floating(f)) if f==3.0);
}

#[test]
fn multibyte_comment(){
    let (value, _) = quick_result("
        ;Grüße aus Lüdinghausen
        (+ 1 1)"
    ).unwrap();
    assert_match!(value, EvalValue::Numeric(Numeric::Integer(i)) if i==2);
}
