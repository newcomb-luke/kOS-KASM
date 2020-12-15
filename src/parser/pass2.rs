use std::{error::Error, slice::Iter, iter::Peekable};

use kerbalobjects::{KOFile};

/// This function performas the second pass of a two-pass assembler.
/// It takes instructions and outputs a KerbalObject file as the result
pub fn pass1(tokens: &Vec<Token>, label_manager: &mut LabelManager) -> Result<KOFile, Box<dyn Error>> {
    
}