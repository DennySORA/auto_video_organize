trait Component {
    fn running(&self);
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
}
