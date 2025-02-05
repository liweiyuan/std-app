use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde::Serialize;
    use std::time::Instant;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        age: u32,
        address: String,
    }

    #[test]
    fn test_write_and_read() {
        // 创建一个Cursor，包装Vec<u8>
        let mut cursor = Cursor::new(Vec::new());
        // 写入数据
        let write_result = cursor.write_all(b"Hello, Rust!");
        assert!(write_result.is_ok());
        assert_eq!(12, cursor.position());
        // 回到开始位置
        let seek = cursor.seek(SeekFrom::Start(0));
        assert!(seek.is_ok());
        // 读取数据
        let mut buffer = String::new();
        let read_result = cursor.read_to_string(&mut buffer);
        assert!(read_result.is_ok());

        assert_eq!(b"Hello, Rust!", buffer.as_bytes());
    }

    #[test]
    //写入不同的位置
    fn test_seek() {
        let mut cursor = Cursor::new(vec![0; 8]);
        let leek = cursor.seek(SeekFrom::Start(4));
        assert!(leek.is_ok());
        let wriet_result = cursor.write_all(&[0xFF]);
        assert!(wriet_result.is_ok());

        let leek = cursor.seek(SeekFrom::Start(2));
        assert!(leek.is_ok());
        let wriet_result = cursor.write_all(&[0xAA]);
        assert!(wriet_result.is_ok());

        //获取底层数据
        let data = cursor.into_inner();
        println!("{:?}", data);
        //assert_eq!([0, 0, 170, 0, 255, 0, 0, 0], data);
    }

    #[test]
    //序列化与反序列化
    fn test_serialize_and_deserialize() -> std::io::Result<()> {
        let person = Person {
            name: "John Doe".to_string(),
            age: 30,
            address: "123 Main St".to_string(),
        };

        //序列化到内存
        let mut buffer = Cursor::new(Vec::new());
        let serialize_result = serde_json::to_writer(&mut buffer, &person)?;
        assert_eq!((), serialize_result);
        //设置到开始位置
        let seek_result = buffer.seek(SeekFrom::Start(0))?;
        assert_eq!(0, seek_result);

        //反序列化
        let john: Person = serde_json::from_reader(buffer)?;
        assert_eq!(john, person);

        Ok(())
    }

    #[test]
    //高效的数据处理管道
    fn test_pipeline() -> std::io::Result<()> {
        //模拟数据源
        let data_source = b"Hello World!".to_vec();
        let mut cursor = Cursor::new(data_source);

        //管道
        let mut buffer = Vec::new();
        {
            let mut writer = Cursor::new(&mut buffer);
            let mut chunk = [0; 4];
            while let Ok(n) = cursor.read(&mut chunk) {
                if n == 0 {
                    break;
                }

                //转为大写
                let upper = chunk[..n]
                    .iter()
                    .map(|&b| b.to_ascii_uppercase())
                    .collect::<Vec<_>>();

                writer.write_all(&upper)?;
            }
        }
        assert_eq!(b"HELLO WORLD!".to_vec(), buffer);
        Ok(())
    }

    //预分配缓冲池
    #[test]
    fn test_preallocate_buffer() -> std::io::Result<()> {
        let mut buffer = Vec::with_capacity(1024);
        let mut cursor = Cursor::new(&mut buffer);

        //写入数据
        for i in 0..10000 {
            writeln!(cursor, "Line {}", i)?;
        }

        println!("Final size: {}", cursor.get_ref().len());
        println!("Capacity: {}", cursor.get_ref().capacity());
        Ok(())
    }

    //零拷贝
    #[test]
    fn test_zero_copy() -> std::io::Result<()> {
        let cursor = Cursor::new(b"Large data block".to_vec());

        //获取引用
        let slice = cursor.get_ref();
        let buffer = &slice[..];

        assert_eq!(b"Large data block".to_vec(), buffer);
        Ok(())
    }

    // 性能对比
    #[test]
    fn test_performance_comparison() -> std::io::Result<()> {
        let size = 1_000_000;
        let data = vec![1u8; size];

        // 使用Vec直接操作
        let start = Instant::now();
        let mut vec = Vec::new();
        vec.extend_from_slice(&data);
        let vec_time = start.elapsed();

        // 使用Cursor操作
        let start = Instant::now();
        let mut cursor = Cursor::new(Vec::with_capacity(size));
        cursor.write_all(&data)?;
        let cursor_time = start.elapsed();

        println!("Vec time: {:?}", vec_time);
        println!("Cursor time: {:?}", cursor_time);

        Ok(())
    }
}
