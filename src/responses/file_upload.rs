use rocket::fs::TempFile;

#[derive(FromForm)]
pub struct Upload<'f> {
    pub upload: TempFile<'f>,
}
