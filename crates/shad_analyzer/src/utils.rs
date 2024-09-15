pub(crate) fn result_ref<T>(result: &Result<T, ()>) -> Result<&T, ()> {
    result.as_ref().map_err(|&()| ())
}
