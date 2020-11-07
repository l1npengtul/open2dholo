// TODO: Change to acutal data format


pub struct ThreadMessage{
    msg: MessageType,
    content: String,
}
impl ThreadMessage{
    pub fn new(mt: MessageType, cnt: String) -> Self{
        ThreadMessage {
            msg: mt,
            content: cnt,
        }
    }
    pub fn get_message_type(&self) -> &MessageType {
        &self.msg
    }
    pub fn get_message_content(&self) -> &String {
        &self.content
    }
}

pub enum MessageType{
    DIE,
    SET,
    CLOSE,
}