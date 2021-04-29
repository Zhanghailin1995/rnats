use rand;
use std::cell::Ref;
use std::fmt::Display;

fn drop_static<T: 'static>(t: T) {
    std::mem::drop(t);
}

struct ByteIter<'remainder> {
    remainder: &'remainder [u8]
}

#[derive(Debug)]
struct NumRef<'a>(&'a i32);

// 当我们有一个关于'a泛型的结构体时，我们几乎不会需要写带有&'a mut self的方法。
// 它的含义是“在这个结构体的整个lifetime，这个方法会可变借用它”。
impl<'a> NumRef<'a> {
    // my struct is generic over 'a so that means I need to annotate
    // my self parameters with 'a too, right? (answer: no, not right)
    fn some_method(&'a mut self) {}

    // no more 'a on mut self
    fn some_method2(&mut self) {}

    // above line desugars to
    fn some_method_desugared<'b>(&'b mut self) {}
}

impl<'remainder> ByteIter<'remainder> {
    fn next(&mut self) -> Option<&'remainder u8> {
        if self.remainder.is_empty() {
            None
        } else {
            let byte = &self.remainder[0];
            self.remainder = &self.remainder[1..];
            Some(byte)
        }
    }
}

trait Trait {}

type T1 = Box<dyn Trait>;

type T2 = Box<dyn Trait + 'static>;

// elided
impl dyn Trait {}

// expand
impl dyn Trait + 'static {}

// elided
type T3<'a> = &'a dyn Trait;
// expanded, &'a T requires T: 'a, so inferred as 'a
type T4<'a> = &'a (dyn Trait + 'a);

// elided
type T5<'a> = Ref<'a, dyn Trait>;
// expanded, Ref<'a, T> requires T: 'a, so inferred as 'a
type T6<'a> = Ref<'a, dyn Trait + 'a>;

trait GenericTrait<'a>: 'a {}

// elided
type T7<'a> = Box<dyn GenericTrait<'a>>;
// expanded
type T8<'a> = Box<dyn GenericTrait<'a> + 'a>;

// elided
impl<'a> dyn GenericTrait<'a> {}

// expanded
impl<'a> dyn GenericTrait<'a> + 'a {}

fn dynamic_thread_print(t: Box<dyn Display + Send>) {
    std::thread::spawn(move || {
        println!("{}", t);
    }).join();
}
// 如上等同如下
// fn dynamic_thread_print(t: Box<dyn Display + Send + 'static>) {
//     std::thread::spawn(move || {
//         println!("{}", t);
//     }).join();
// }

// error[E0310]: the parameter type `T` may not live long enough
//   --> src/lib.rs:10:5
//    |
// 9  | fn static_thread_print<T: Display + Send>(t: T) {
//    |                        -- help: consider adding an explicit lifetime bound...: `T: 'static +`
// 10 |     std::thread::spawn(move || {
//    |     ^^^^^^^^^^^^^^^^^^
//    |
// note: ...so that the type `[closure@src/lib.rs:10:24: 12:6 t:T]` will meet its required lifetime bounds
//   --> src/lib.rs:10:5
//    |
// 10 |     std::thread::spawn(move || {
//    |     ^^^^^^^^^^^^^^^^^^
//
// 作者：juu wiio
// 链接：https://zhuanlan.zhihu.com/p/165976086
// 来源：知乎
// 著作权归作者所有。商业转载请联系作者获得授权，非商业转载请注明出处。
fn static_thread_print<T: Display + Send + 'static>(t: T) {
    std::thread::spawn(move || {
        println!("{}", t);
    }).join();
}

// --------------------------- 8---------------------
// 1. lifetime是在编译时静态验证的
// 2. 在运行时，lifetime不可能以任何方式伸长缩短或改变
// 3. Rust的borrow checker总是会假设所有的分支都会走到，为变量选择其中最短的lifetime
struct Has<'lifetime> {
    lifetime: &'lifetime str,
}

// --------------------------- 9---------------------
fn takes_shared_ref(n: &i32) {}

fn main() {
    {
        // --------------------------- 9---------------------
        let mut a = 10;
        takes_shared_ref(&mut a); // compiles
        takes_shared_ref(&*(&mut a)); // above line desugared
    }
    {
        // -------------------------- 8---------------------
        let long = String::from("long");
        let mut has = Has { lifetime: &long };
        assert_eq!(has.lifetime, "long");

        {
            let short = String::from("short");
            // "switch" to short lifetime
            has.lifetime = &short;
            assert_eq!(has.lifetime, "short");

            // "switch back" to long lifetime(but not really)
            has.lifetime = &long;
            assert_eq!(has.lifetime, "long");
            // `short` dropped here
        }

        // compile error, `short` still "borrowed" after drop
        assert_eq!(has.lifetime, "long");
    }


    let mut num_ref = NumRef(&5);
    num_ref.some_method2();
    num_ref.some_method2();
    println!("{:?}", num_ref);

    let mut bytes = ByteIter { remainder: b"1123" };
    let byte_1 = bytes.next();
    let byte_2 = bytes.next();
    std::mem::drop(bytes); // we can even drop the iterator now!
    if byte_1 == byte_2 { // compiles
        // do something
    }

    let mut strings: Vec<String> = Vec::new();

    for _ in 0..10 {
        if rand::random() {
            // all the strings are randomly generated
            // and dynamically allocated at run-time
            let string = rand::random::<u64>().to_string();
            strings.push(string);
        }
    }

    // strings are owned types so they're bounded by 'static
    // 这个迭代会move掉strings
    for mut string in strings {
        // all the strings are mutable
        string.push_str("a mutation");
        // all the strings are droppable
        drop_static(string); // compiles
    }

    // all the strings have been invalidated before the end of the program
    println!("i am the end of the program");

    //strings.iter().map(|s| println!("{}", s));
}