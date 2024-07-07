pub trait Write {
    fn write(&mut self, buf: &[u8]) -> anyhow::Result<usize>;
    fn flush(&mut self) -> anyhow::Result<()>;
}
