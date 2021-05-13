#[derive(Copy, Clone)]
struct FooCopy {
    value: i32,
}

impl FooCopy {
    fn new(value: i32) -> Self {
        Self { value }
    }

    fn get(&self) -> i32 {
        self.value
    }

    fn increase(&mut self) {
        self.value += 1;
    }
}

fn is_fn_mut<F: FnMut()>(_closure: &F) {}

fn is_copy<F: Copy>(_closure: &F) {}


fn main() {
    let mut foo_copy = FooCopy::new(0);

    // 注意，在标准库文档和 The Rust Reference 中都明确说明了闭包实现FnOnce、FnMut和Fn中
    // 的哪个trait只与闭包如何使用所捕获的变量有关，与如何捕获变量无关。关键字move影响的是闭
    // 包如何捕获变量，因此，对闭包实现FnOnce、FnMut和Fn没有任何影响
    //
    // 闭包是否实现Copy trait，只与捕获的变量是否可以被copy有关，与如何使用（是否修改捕获的
    // 变量）无关。
    //
    // 作者：SneakyCat
    // 链接：https://zhuanlan.zhihu.com/p/341815515
    // 来源：知乎
    // 著作权归作者所有。商业转载请联系作者获得授权，非商业转载请注明出处。
    let mut c_with_move = move || {
        for _ in 0..5 {
            foo_copy.increase(); // 闭包对变量进行了修改，即通过可变借用使用所捕获的变量，则会实现FnMut
            // FooCopy实现了Copy，且闭包使用了move，则闭包实现了Copy
        }

        println!("foo_copy in closure(with move): {}", foo_copy.get());
    };

    c_with_move();
    println!("foo_copy out of closure: {}\n", foo_copy.get());

    let mut c_without_move = || {
        for _ in 0..5 {
            foo_copy.increase(); // 闭包对变量进行了修改，即通过可变借用使用所捕获的变量，则会实现FnMut
            // 但是闭包并没有使用move，而是以&mut T的形式捕获了变量，&mut T是Move语义，所以闭包不会实现Copy
        }

        println!("foo_copy in closure(without move): {}", foo_copy.get());
    };

    is_fn_mut(&c_with_move);
    is_copy(&c_with_move);

    is_fn_mut(&c_without_move);
    //is_Copy(&c_without_move); // Error

    c_without_move();
    println!("foo_copy out of closure(without move): {}\n", foo_copy.get());

    c_with_move();
    println!("foo_copy out of closure(with move): {}\n", foo_copy.get());
}

