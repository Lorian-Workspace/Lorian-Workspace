fn main() {
    // Solo compilar el recurso en Windows
    #[cfg(windows)]
    {
        if let Err(e) = winres::WindowsResource::new()
            .set_icon("icon.ico")
            .compile()
        {
            // Si no se puede cargar el icono personalizado, continuar sin él
            eprintln!("Warning: No se pudo cargar el icono personalizado: {}", e);
            eprintln!("La aplicación funcionará con el icono por defecto.");
        }
    }
}
