use std::io;
use rand::Rng;
use std::cmp::Ordering;


fn main() {
    println!("猜数字游戏~");
    let secret_number = rand::thread_rng().gen_range(1, 101);
    // println!("生成的随机数是：{}", secret_number);

    loop {
        println!("输入数字：");
        let mut guess = String::new();
        io::stdin().read_line(&mut guess)
            .expect("读取数据失败...");
        // parse()返回一个Result类型，这是个枚举类型，有Ok和Err两个值
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("你输入的数据是：{}", guess);

        // 模式匹配器
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("猜小了！"), 
            Ordering::Greater => println!("猜大了！"), 
            Ordering::Equal => {
                println!("猜对了！");
                break; // 打断外层loop循环
            }
        }
    }
}


