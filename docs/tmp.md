```java
public class CompositePackResources
implements PackResourcesTrait {
    private final PackResources primaryPackResources;
    private final List<PackResources> packResourcesStack;
```

```rust
pub struct CompositePackResources {
    primary: PackResources,
    stack: Vec<PackResources>,
}
pub enum PackResources {
    Path(PathPackResources),
    File(FilePackResources),
    Vanilla(VanillaPackResources),
    Composite(CompositePackResources)
}
```