use jclassfile::{class_file::ClassFile, constant_pool::ConstantPool, methods::MethodFlags};

#[derive(Debug)]
pub struct Classfile {
    pub fqdn: String,
    pub methods: Vec<Method>,
    // TODO: support inner classes / records / interfaces  etc.
}

#[derive(Debug)]
pub struct Method {
    pub flags: MethodFlags,
    pub name: String,
    pub parameters: Vec<JavaType>,
    pub return_type: JavaType,
}

#[derive(Debug)]
pub enum JavaType {
    Void,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
    Boolean,
    Class(String),
    Array(Box<JavaType>),
}

impl Classfile {
    pub fn new(bytes: &[u8]) -> Option<Self> {
        let class = jclassfile::class_file::parse(bytes).ok()?;

        Some(Self {
            fqdn: parse_class_name(&class)?.replace("/", "."),
            methods: parse_methods(&class)?,
        })
    }
}

fn parse_methods(class: &ClassFile) -> Option<Vec<Method>> {
    let mut methods = Vec::new();

    for method in class.methods() {
        let descriptor = parse_string(class, method.descriptor_index() as usize)?;
        // TODO: Can we assume that the descriptor will always be valid and start with a '('
        let mut descriptor_iter = descriptor.chars().skip(1);
        let mut parameters = Vec::new();
        let mut return_type = None;

        while let Some(char) = descriptor_iter.next() {
            if char == ')' {
                return_type = parse_field(descriptor_iter.next(), &mut descriptor_iter);
                break;
            } else {
                parameters.push(parse_field(Some(char), &mut descriptor_iter)?);
            }
        }

        methods.push(Method {
            flags: MethodFlags::from_bits(method.access_flags().bits())?,
            name: parse_string(class, method.name_index() as usize)?,
            parameters,
            return_type: return_type?,
        });
    }

    Some(methods)
}

fn parse_field(c: Option<char>, chars: &mut impl Iterator<Item = char>) -> Option<JavaType> {
    match c {
        Some('B') => Some(JavaType::Byte),
        Some('C') => Some(JavaType::Char),
        Some('D') => Some(JavaType::Double),
        Some('F') => Some(JavaType::Float),
        Some('I') => Some(JavaType::Int),
        Some('J') => Some(JavaType::Long),
        Some('L') => Some(JavaType::Class(
            chars
                .take_while(|char| char != &';')
                .collect::<String>()
                .replace('/', "."),
        )),
        Some('S') => Some(JavaType::Short),
        Some('Z') => Some(JavaType::Boolean),
        Some('[') => Some(JavaType::Array(Box::new(parse_field(chars.next(), chars)?))),
        Some('V') => Some(JavaType::Void),
        _ => None,
    }
}

fn parse_class_name(class: &ClassFile) -> Option<String> {
    let name_index = match class.constant_pool().get(class.this_class() as usize)? {
        ConstantPool::Class { name_index } => Some(*name_index as usize),
        _ => None,
    }?;
    parse_string(class, name_index)
}

fn parse_string(class: &ClassFile, index: usize) -> Option<String> {
    match class.constant_pool().get(index)? {
        ConstantPool::Utf8 { value } => Some(value.clone()),
        _ => None,
    }
}
