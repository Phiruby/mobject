/// This specifies the kind of pipeline the mobject needs to be rendered
/// Each pipeline has their own required descriptor set layout that needs to be
/// adhered. Each mobject implementing a specific pipeline is responsible
/// to follow this layout.
enum Pipelines {
    Flat2D,
    Textured2D,
    Space3D
}
