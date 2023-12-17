//use crate::prelude::*;

pub mod thecodeatom;
pub mod thecodebundle;
pub mod thecodeeditor;
pub mod thecodefunction;
pub mod thecodegrid;
pub mod thecodemodule;
pub mod thecodenode;
pub mod thecodeobject;
pub mod thecodesandbox;
pub mod thecompiler;

pub mod prelude {
    pub use crate::thecode::thecodeatom::{TheCodeAtom, TheCodeAtomKind};
    pub use crate::thecode::thecodebundle::TheCodeBundle;
    pub use crate::thecode::thecodeeditor::TheCodeEditor;
    pub use crate::thecode::thecodefunction::TheCodeFunction;
    pub use crate::thecode::thecodegrid::{
        TheCodeGrid, TheCodeGridMessage, TheCodeGridMessageType,
    };
    pub use crate::thecode::thecodemodule::TheCodeModule;
    pub use crate::thecode::thecodenode::{TheCodeNode, TheCodeNodeCall};
    pub use crate::thecode::thecodeobject::TheCodeObject;
    pub use crate::thecode::thecodesandbox::TheCodeSandbox;
    pub use crate::thecode::thecompiler::{TheCompiler, TheCompilerContext, TheCompilerError};
}

pub struct TheReturnCode {}
//    fn new() -> Self
//}
