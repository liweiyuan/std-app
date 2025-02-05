macro_rules! say_hello {
    () => {
        println!("Hello, world!");
    };

    //带参数
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

// 通过宏来构建vec
macro_rules! vector {
     // 空向量
     () => {
        Vec::<_>::new()
    };
    // 多个元素或单个元素
    ($($name:expr),+ $(,)?) => {{
        vec![$($name),+]
    }};
}

//递归宏
macro_rules! calc {
    ($num:expr) => {
        $num
    };
    //add
    ($num:expr, add $($rest:tt)*) =>{
        calc!($num + $($rest)*)
    };
    //sub
    ($num:expr, sub $($rest:tt)*) => {
        calc!($num - $($rest)*)
    };
}

//类型构建宏
macro_rules! create_struct {
    ($struct_name:ident {$($field_name:ident : $field_type:ty),*  $(,)? }) => {
        #[derive(Debug, Clone, Default)]
        struct $struct_name{
            $($field_name: $field_type),*
        }

        impl $struct_name {
            fn new($($field_name: $field_type),*) -> Self {
                $struct_name {
                    $($field_name),*
                }
            }

            fn default()-> Self {
                Default::default()
            }
        }
    };
}

macro_rules! ensure {
    ($condition:expr, $err:expr) => {
        if !$condition {
            return Err($err.into());
        }
    };
}

macro_rules! try_or_return {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err.into()),
        }
    };
}

#[derive(Debug, PartialEq)]
struct MyError(String);

impl From<&str> for MyError {
    fn from(s: &str) -> Self {
        MyError(s.to_string())
    }
}

fn check_positive(num: i32) -> Result<i32, MyError> {
    ensure!(num > 0, "Number must be positive");
    Ok(num)
}

#[cfg(test)]
mod test_macro {
    use super::*;

    #[test]
    fn test_say_hello() {
        say_hello!();
        say_hello!("Alice");
    }

    #[test]
    fn test_vector() {
        let x: Vec<i32> = vector!();
        assert_eq!(x, Vec::<i32>::new());
        assert_eq!(vector!(1), vec![1]);
        assert_eq!(vector!(1, 2, 3), vec![1, 2, 3]);
    }

    #[test]
    fn test_calc() {
        assert_eq!(calc!(10, add 5, sub 3), 12);
    }

    //类型构建宏测试
    #[test]
    fn test_create_struct() {
        create_struct!(Person {
            name: String,
            age: u32,
            address: String
        });

        // 使用带参数的构造函数
        let person = Person::new("John".to_string(), 30, "john@example.com".to_string());

        assert_eq!("John", person.name);
        assert_eq!(30, person.age);
        assert_eq!("john@example.com", person.address);

        // 使用默认构造函数
        let default_person = Person::default();
        assert_eq!(default_person.name, "");
        assert_eq!(default_person.age, 0);
    }

    #[test]
    fn test_ensure_pass() {
        let result = check_positive(5);
        assert_eq!(result, Ok(5)); // 当 num > 0 时，应该返回 Ok(5)
    }

    #[test]
    fn test_ensure_fail() {
        let result = check_positive(-5);
        assert_eq!(result, Err(MyError("Number must be positive".into()))); // 当 num <= 0 时，应该返回 Err
    }

    fn process_number(num: i32) -> Result<i32, MyError> {
        let closure = || {
            if num > 0 {
                Ok(num * 2)
            } else {
                Err("Negative number")
            }
        };
        let x = try_or_return!(closure());
        Ok(x + 10)
    }

    #[test]
    fn test_try_or_return() {
        let result = process_number(3);
        assert_eq!(result, Ok(16));

        let result = process_number(-1);
        assert_eq!(result, Err(MyError("Negative number".into()))); // 处理错误
    }
}
