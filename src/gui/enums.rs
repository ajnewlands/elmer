/// Hold the possible outcomes of a modal. Note that 'None'
/// also is an option in an immediate mode GUI.
pub enum ModalResult {
    None,
    Ok,
    Cancel,
}
