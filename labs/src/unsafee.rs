#[cfg(test)]
mod tests {

    #[test]
    fn test_unsafe_use_after_free() {
        let x = Box::new(42);
        let ptr = Box::into_raw(x); // получаем сырой указатель

        unsafe {
            // освобождаем память вручную
            Box::from_raw(ptr);
            // снова используем ptr, хотя память уже освобождена
            dbg!(*ptr); // use-after-free
        }
    }

    #[test]
    fn test_unsafe_double_free() {
        let x = Box::new(10);
        let ptr = Box::into_raw(x);

        unsafe {
            // Первый раз освобождаем
            Box::from_raw(ptr);
            // Второй раз — undefined behavior (double free)
            Box::from_raw(ptr);
        }
    }
}