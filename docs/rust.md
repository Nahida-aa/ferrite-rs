从设计层面避免 `dyn` 的核心策略 `dyn` 的出现通常是因为运行时类型不确定。设计时消除这种不确定性，就能避免 `dyn`。 

--- 
### 1. 用泛型替代运行时多态 ❌ 需要 `dyn` 的设计 
```rust 
// 运行时才知道要渲染什么 
fn render_all(items: &[Box<dyn Drawable>]) { 
    for item in items { item.draw(); } 
} 
let scene = vec![ 
    Box::new(Circle) as Box<dyn Drawable>, 
    Box::new(Rectangle), 
]; 
```
✅ 用泛型重新设计 
```rust 
// 编译期确定类型，场景由单一类型构成 
struct Scene<T: Drawable> { items: Vec<T>, } 
impl<T: Drawable> Scene<T> { 
    fn render(&self) { 
        for item in &self.items { 
            item.draw(); 
            // 静态分发 
        }
    } 
} 
// 使用 
let circle_scene = Scene::<Circle> { items: vec![...] }; 
let rect_scene = Scene::<Rectangle> { items: vec![...] }; 
``` 
关键改变：不是"一个场景混存多种形状"，而是"一种场景只存一种形状"。 

--- 
### 2. 用枚举封闭类型集合 如果类型集合固定且已知： 
```rust 
// ❌ dyn 方式 
let shapes: Vec<Box<dyn Drawable>> = vec![...]; 
// ✅ 枚举方式 
enum Shape { 
    Circle(Circle), 
    Rectangle(Rectangle), 
    Triangle(Triangle), 
} 
impl Drawable for Shape { 
    fn draw(&self) { 
        match self { 
            Shape::Circle(c) => c.draw(), 
            Shape::Rectangle(r) => r.draw(), 
            Shape::Triangle(t) => t.draw(), 
        }
    } 
} 
let shapes: Vec<Shape> = vec![ Shape::Circle(Circle), Shape::Rectangle(Rectangle), ]; 
```
设计前提：所有可能的类型在开发时就确定，不会扩展。 
 
--- 
### 3. 用组合替代继承式抽象 ❌ 面向对象思维（导致 dyn）

```rust 
trait Animal { fn speak(&self); } 
struct Dog; struct Cat; 
// 需要 Vec<dyn Animal> 来混存 
let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)]; 
``` 
✅ 组合思维（避免 dyn） 
```rust 
// 不需要抽象 Animal，直接描述场景 
struct PetShop { 
    dogs: Vec<Dog>, 
    cats: Vec<Cat>, 
} 
impl PetShop { 
    fn make_noise(&self) { 
        for dog in &self.dogs { 
            dog.bark(); 
        } 
        for cat in &self.cats { 
            cat.meow(); 
        } 
    } 
} 
``` 
设计转变：不是"动物列表"，而是"狗列表 + 猫列表"。 

--- 
### 4. 用 ID 和类型标签替代类型擦除 
```rust 
// ❌ dyn 方式 
trait Component { fn update(&self); } 
let components: Vec<Box<dyn Component>> = vec![...]; 
// ✅ ID + 枚举方式 
#[derive(Clone, Copy, PartialEq, Eq)] 
enum ComponentType { Physics, Render, Input, } 
struct Entity { 
    id: u64, 
    components: 
    HashMap<ComponentType, ComponentId>, 
} 
// 每种组件单独存储，通过 ID 关联 
struct World { 
    physics: Vec<PhysicsComponent>, 
    renders: Vec<RenderComponent>, 
} 

``` 
ECS 架构：Bevy 的核心设计，完全避免 `dyn`，用 ID 和数组索引实现多态。 

--- 

5. 用构建器模式延迟类型确定 

```rust 
// ❌ 运行时选择类型 
fn create_shape(kind: &str) -> Box<dyn Shape> { 
    match kind { 
        "circle" => Box::new(Circle), 
        "rect" => Box::new(Rectangle), 
        _ => panic!(), 
    } 
} 
// ✅ 编译期通过泛型参数确定 
struct Game<S: Shape> { player_shape: S, } 
// 不同类型创建不同游戏实例 
let game1 = Game::<Circle> { 
    player_shape: Circle 
}; 
let game2 = Game::<Rectangle> { 
    player_shape: Rectangle 
}; 
``` 

--- 
### 6. 用宏生成代码替代运行时抽象 

```rust 
// ❌ dyn 方式：运行时遍历不同类型的处理器 
trait Handler { fn handle(&self, req: Request); } 
let handlers: Vec<Box<dyn Handler>> = vec![...]; 
// ✅ 宏方式：编译期生成匹配代码 
macro_rules! define_router { 
    ($($path:literal => $handler:ty),*) => { 
        fn route(path: &str, req: Request) { 
            match path { 
                $( $path => <$handler>::handle(req), )* _ => not_found(req), 
            } 
        } 
    }; 
} 
define_router! { 
    "/users" => UserHandler, 
    "/posts" => PostHandler 
} 
``` 
---
设计层面的核心原则 
| 原则 | 说明 | 
| - | - |
| 编译期确定 > 运行时确定 | 用泛型、枚举、宏把决策提前到编译期 |
| 同构集合 > 异构集合 | 一个容器只存一种类型，用多个容器替代混存 |
| 组合 > 继承式抽象 | 描述"有什么"，不是"是什么" |
| ID 关联 > 类型嵌套 | ECS 模式，用索引替代指针  |
|代码生成 > 运行时分发 | 宏、模板、构建脚本生成具体代码  |

--- 
### 一句话 
> 避免 `dyn` 的关键是设计时消除运行时类型不确定性：用泛型让类型编译期确定，用枚举封闭类型集合，用组合替代抽象，用 ID 和数组索引替代指针多态。`dyn` 不是敌人，但设计层面优先用静态分发是 Rust 的哲学。