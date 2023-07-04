


//! # Intro
//!
//! This document covers the usage of the crate's macros, it does 
//! not delve into the detailed logic of the generated code.
//! 
//! For a comprehensive understanding of the underlying
//! concepts and implementation details of the Actor Model,  
//! it's recommended to read the article  [Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/)
//! by Alice Ryhl ( also known as _Darksonn_ ) also a great 
//! talk by the same author on the same subject if a more 
//! interactive explanation is prefered 
//! [Actors with Tokio – a lesson in ownership - Alice Ryhl](https://www.youtube.com/watch?v=fTXuGRP1ee4)
//! (video).
//! This article not only inspired the development of the 
//! `interthread` crate but also serves as foundation 
//! for the Actor Model implementation logic in it. 


//! ## What is an Actor ?
//!
//! Despite being a fundamental concept in concurrent programming,
//! defining exactly what an actor is can be ambiguous.
//! 
//! - *Carl Hewitt*, often regarded as the father of the Actor Model,
//! [The Actor Model](https://www.youtube.com/watch?v=7erJ1DV_Tlo) (video).
//! 
//! - Wikipidia [Actor Model](https://en.wikipedia.org/wiki/Actor_model)
//!  
//!
//! a quote from [Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/):
//! 
//! > "The basic idea behind an actor is to spawn a 
//! self-contained task that performs some job independently
//! of other parts of the program. Typically these actors
//! communicate with the rest of the program through 
//! the use of message passing channels. Since each actor 
//! runs independently, programs designed using them are 
//! naturally parallel."
//! > - Alice Ryhl 
//!
//! ## What is the problem ?
//! 
//! To achieve parallel execution of individual objects 
//! within the same program, it is challenging due 
//! to the need for various types that are capable of 
//! working across threads. The main difficulty 
//! lies in the fact that as you introduce thread-related types,
//! you can quickly lose sight of the main program 
//! idea as the focus shifts to managing thread-related 
//! concerns.
//!
//! It involves using constructs like threads, locks, channels,
//! and other synchronization primitives. These additional 
//! types and mechanisms introduce complexity and can obscure 
//! the core logic of the program.
//! 
//! 
//! Moreover, existing libraries like [`actix`](https://docs.rs/actix/latest/actix/), [`axiom`](https://docs.rs/axiom/latest/axiom/), 
//! designed to simplify working within the Actor Model,
//! often employ specific concepts, vocabulary, traits and types that may
//! be unfamiliar to users who are less experienced with 
//! asynchronous programming and futures. 
//! 
//! ## Solution 
//! 
//! The [`actor`](./attr.actor.html) macro -  when applied to the 
//! implementation block of a given "MyActor" object,
//! generates additional types and functions 
//! that enable communication between threads.
//! 
//! A notable outcome of applying this macro is the 
//! creation of the `MyActorLive` struct ("ActorName" + "Live"),
//! which acts as an interface/handle to the `MyActor` object.
//! `MyActorLive` retains the exact same public method signatures
//! as `MyActor`, allowing users to interact with the actor as if 
//! they were directly working with the original object.
//! 
//! ### Examples
//! 
//! 
//! Filename: Cargo.toml
//! 
//!```text
//![dependencies]
//!interthread = "0.2.0"
//!oneshot     = "0.1.5" 
//!```
//! 
//! Filename: main.rs
//!```rust
//!pub struct MyActor {
//!    value: i8,
//!}
//!
//!#[interthread::actor(channel=2)] // <-  this is it 
//!impl MyActor {
//!
//!    pub fn new( v: i8 ) -> Self {
//!       Self { value: v } 
//!    }
//!    pub fn increment(&mut self) {
//!        self.value += 1;
//!    }
//!    pub fn add_number(&mut self, num: i8) -> i8 {
//!        self.value += num;
//!        self.value
//!    }
//!    pub fn get_value(&self) -> i8 {
//!        self.value
//!    }
//!}
//! // uncomment to see the generated code
//! //#[interthread::example(file="src/main.rs")]  
//!fn main() {
//!
//!    let actor = MyActorLive::new(5);
//!
//!    let mut actor_a = actor.clone();
//!    let mut actor_b = actor.clone();
//!
//!    let handle_a = std::thread::spawn( move || { 
//!    actor_a.increment();
//!    });
//!
//!    let handle_b = std::thread::spawn( move || {
//!    actor_b.add_number(5);
//!    });
//!
//!    let _ = handle_a.join();
//!    let _ = handle_b.join();
//!
//!    assert_eq!(actor.get_value(), 11)
//!}
//!
//! ```
//! 
//! An essential point to highlight is that when invoking 
//! `MyActorLive::new`, not only does it return an instance 
//! of `MyActorLive`, but it also spawns a new thread that 
//! contains an instance of `MyActor` in it. 
//! This introduces parallelism to the program.
//! 
//! The code generated by the [`actor`](./attr.actor.html) takes 
//! care of the underlying message routing and synchronization, 
//! allowing developers to rapidly prototype their application's
//! core functionality. This fast sketching capability is
//! particularly useful when exploring different design options, 
//! experimenting with concurrency models, or implementing 
//! proof-of-concept systems. Not to mention, the cases where 
//! the importance of the program lies in the result of its work 
//! rather than its execution.
//!
//! 
//! # SDPL Framework
//! 
//! 
//!  The code generated by the [`actor`](./attr.actor.html) macro 
//! can be divided into four more or less important but distinct 
//! parts: [`script`](#script) ,[`direct`](#direct), 
//! [`play`](#play), [`live`](#live) .
//! 
//!  This categorization provides an intuitive 
//! and memorable way to understand the different aspects 
//! of the generated code.
//! 
//! Expanding the above example, uncomment the [`example`](./attr.example.html)
//! placed above the `main` function, go to `examples/inter/main.rs` in your 
//! root directory and find `MyActor` along with additional SDPL parts :
//! 
//! # `script`
//! 
//!  Think of script as a message type definition.
//! 
//!  The declaration of an `ActorName + Script` enum, which is 
//! serving as a collection of variants that represent 
//! different messages that may be sent across threads through a
//! channel. 
//! 
//!  Each variant corresponds to a struct with fields
//! that capture the input and/or output parameters of 
//! the respective public methods of the Actor.
//!  
//! 
//! ```rust
//! 
//!#[derive(Debug)]
//!pub enum MyActorScript {
//!    Increment {},
//!    AddNumber {
//!        input: (i8),
//!        output: oneshot::Sender<i8>,
//!    },
//!    GetValue {
//!        output: oneshot::Sender<i8>,
//!    },
//!}
//! 
//! ```
//! 
//! > **Note**: Method `new` not included as a variant in the `script`. 
//! 
//! 
//! # direct
//! The implementation block of [`script`](#script), specifically 
//! the `actor_name_ + direct` method which allows 
//! for direct invocation of the Actor's methods by mapping 
//! the enum variants to the corresponding function calls.
//! 
//! 
//! ```rust
//!impl MyActorScript {
//!    pub fn my_actor_direct(self, actor: &mut MyActor) {
//!        match self {
//!            MyActorScript::Increment {} => {
//!                actor.increment();
//!            }
//!            MyActorScript::AddNumber {
//!                input: (num),
//!                output: send,
//!            } => {
//!                send.send(actor.add_number(num))
//!                    .expect("'my_actor_direct.send'. Channel closed");
//!            }
//!            MyActorScript::GetValue { output: send } => {
//!                send.send(actor.get_value())
//!                    .expect("'my_actor_direct.send'. Channel closed");
//!            }
//!        }
//!    }
//!}
//! 
//! ```
//! 
//! # play
//! The function  `actor_name_ + play` responsible for 
//! continuously receiving `script` variants from 
//! a dedicated channel and `direct`ing them.
//! 
//! Also this function serves as the home for the Actor itself.
//! 
//! 
//!```rust
//!pub fn my_actor_play(
//!    receiver: std::sync::mpsc::Receiver<MyActorScript>, 
//!    mut actor: MyActor) {
//! 
//!    while let Ok(msg) = receiver.recv() {
//!        msg.my_actor_direct(&mut actor);
//!    }
//!    eprintln!("MyActor end of life ...");
//!}
//!``` 
//! 
//! When using the [`edit`](./attr.actor.html#edit) argument in the [`actor`](./attr.actor.html) 
//! macro, such as 
//! 
//!```rust
//!#[interthread::actor(channel=2, edit(play))]
//!``` 
//! 
//! it allows for manual implementation of the `play` part, which 
//! gives the flexibility to customize and modify 
//! the behavior of the `play` to suit any requared logic.
//! 
//! 
//! # live
//! A struct `ActorName + Live`, which serves as an interface/handler 
//! replicating the public method signatures of the original Actor.
//! 
//! Invoking a method on a live instance, it's triggering the eventual 
//! invocation of the corresponding method within the Actor. 
//! 
//! The special method of `live` method `new`  
//! - declares a new channel
//! - initiates an instace of the Actor
//! - spawns the `play` component in a separate thread 
//! - returns an instance of `Self`
//! 
//! 
//! ```rust 
//! 
//!impl MyActorLive {
//!    pub fn new(v: i8) -> Self {
//!        let (sender, receiver) = std::sync::mpsc::sync_channel(2);
//!        let actor = MyActor::new(v);
//!        let actor_live = Self { sender };
//!        std::thread::spawn(|| my_actor_play(receiver, actor));
//!        actor_live
//!    }
//!    pub fn increment(&mut self) {
//!        let msg = MyActorScript::Increment {};
//!        let _ = self
//!            .sender
//!            .send(msg)
//!            .expect("'MyActorLive::method.send'. Channel is closed!");
//!    }
//!    pub fn add_number(&mut self, num: i8) -> i8 {
//!        let (send, recv) = oneshot::channel();
//!        let msg = MyActorScript::AddNumber {
//!            input: (num),
//!            output: send,
//!        };
//!        let _ = self
//!            .sender
//!            .send(msg)
//!            .expect("'MyActorLive::method.send'. Channel is closed!");
//!        recv.recv()
//!            .expect("'MyActorLive::method.recv'. Channel is closed!")
//!    }
//!    pub fn get_value(&self) -> i8 {
//!        let (send, recv) = oneshot::channel();
//!        let msg = MyActorScript::GetValue { output: send };
//!        let _ = self
//!            .sender
//!            .send(msg)
//!            .expect("'MyActorLive::method.send'. Channel is closed!");
//!        recv.recv()
//!            .expect("'MyActorLive::method.recv'. Channel is closed!")
//!    }
//!}
//! 
//! ```
//! The methods of `live` type have same method signature
//! as Actor's own methods 
//! - declare a `oneshot` channel
//! - declare a `msg` specific `script` variant
//! - send the `msg` via `live`'s channel 
//! - receive and return the output if any   
//! 
//! # Panics
//! 
//! If the types used for input or output for actor methods 
//! do not implement the `Send`, `Sync`, and `Debug` traits.
//! 
//! Additionally, the actor object itself should implement 
//! the `Send` trait, allowing it to be safely moved 
//! to another thread for execution. 
//! 
//! # Macro Implicit Dependencies
//!
//! The [`actor`](./attr.actor.html) macro generates code
//! that utilizes channels for communication. However, 
//! the macro itself does not provide any channel implementations.
//! Therefore, depending on the libraries used in your project, 
//! you may need to import additional crates.
//!
//!### Crate Compatibility
//!<table>
//!  <thead>
//!    <tr>
//!      <th>lib</th>
//!      <th><a href="https://docs.rs/oneshot">oneshot</a></th>
//!      <th><a href="https://docs.rs/async-channel">async_channel</a></th>
//!    </tr>
//!  </thead>
//!  <tbody>
//!    <tr>
//!      <td>std</td>
//!      <td style="text-align: center;">&#10003;</td>
//!      <td style="text-align: center;"><b>-</b></td>
//!    </tr>
//!    <tr>
//!      <td><a href="https://crates.io/crates/smol">smol</a></td>
//!      <td style="text-align: center;">&#10003;</td>
//!      <td style="text-align: center;">&#10003;</td>
//!    </tr>
//!    <tr>
//!      <td><a href="https://docs.rs/tokio">tokio</a></td>
//!      <td style="text-align: center;"><b>-</b></td>
//!      <td style="text-align: center;"><b>-</b></td>
//!    </tr>
//!    <tr>
//!      <td><a href="https://crates.io/crates/async-std">async-std</a></td>
//!      <td style="text-align: center;">&#10003;</td>
//!      <td style="text-align: center;"><b>-</b></td>
//!    </tr>
//!  </tbody>
//!</table>
//!
//! 
//!>**Note:** The table shows the compatibility of 
//!>the macro with different libraries, indicating whether 
//!>the dependencies are needed (✔) or not. 
//!>The macros will provide helpful messages indicating 
//!>the necessary crate imports based on your project's dependencies.
 

mod attribute;
mod use_macro;
mod show;
mod file;
mod actor_gen;
mod name;
mod method;
mod check;
mod error;


static INTERTHREAD: &'static str            = "interthread";
static INTER_EXAMPLE_DIR_NAME: &'static str = "INTER_EXAMPLE_DIR_NAME";
static INTER: &'static str                  = "inter";
static GROUP: &'static str                  = "group";
static ACTOR: &'static str                  = "actor";
static EXAMPLE: &'static str                = "example";
static EXAMPLES: &'static str               = "examples";


/// # Code transparency and exploration
///  
/// The [`example`](./attr.example.html) macro serves as a 
/// convenient tool for code transparency and exploration.
/// Automatically generating an expanded code file,
/// it provides developers with a tangible representation of
/// the code produced by the `interthread` macros. 
/// 
/// Having the expanded code readily available in the `examples/inter`
/// directory offers a few key advantages:
///  
/// - It provides a clear reference point for developers to inspect 
/// and understand the underlying code structure.
/// 
/// - The generated code file serves as a starting point for 
/// customization. Developers can copy and paste the generated code 
/// into their own project files and make custom changes as needed. 
/// This allows for easy customization of the generated actor 
/// implementation to fit specific requirements or to add additional 
/// functionality.
/// 
/// - Helps maintain a clean and focused project structure, 
/// with the `examples` directory serving as a dedicated location for 
/// exploring and experimenting with the generated code.
/// 
/// [`example`](./attr.example.html) macro helps developers to 
/// actively engage with the generated code 
/// and facilitates a smooth transition from the generated code to a 
/// customized implementation. This approach promotes code transparency,
/// customization, and a better understanding of the generated code's 
/// inner workings, ultimately enhancing the development experience 
/// when working with the `interthread` macros.
/// 
/// Consider a macro [`actor`](./attr.actor.html)  inside the project 
/// in `src/my_file.rs`.
/// 
///Filename: my_file.rs 
///```rust
///use interthread::{actor,example};
///
///pub struct Number;
///
/// // you can have "example" macro in the same file
/// // #[example(file="src/my_file.rs")]
///
///#[actor(channel=5)]
///impl Number {
///    pub fn new(value: u32) -> Self {Self}
///}
///
///```
/// 
///Filename: main.rs 
///```rust
///use interthread::example;
///#[example(file="src/my_file.rs")]
///fn main(){
///}
///
///```
/// 
/// The macro will create and write to `examples/inter/my_file.rs`
/// the content of `src/my_file.rs` with the 
/// [`actor`](./attr.actor.html) macro expanded.
/// 
/// 
///```text
///my_project/
///├── src/
///│  ├── my_file.rs      <---  macro "actor" 
///|  |
///│  └── main.rs         <---  macro "example" 
///|
///├── examples/          
///   ├── ...
///   └── inter/      
///      ├── my_file.rs   <--- expanded "src/my_file.rs"  
///```
///
/// [`example`](./attr.example.html) macro can be placed on any 
/// item in any file within your `src` directory, providing 
/// flexibility in generating example code for/from different 
/// parts of your project.
///
/// It provides two options for generating example code files: 
///  - [`mod`](##mod)  (default)
///  - [`main`](##main) 
///
/// ## mod 
/// The macro generates an example code file within the 
/// `examples/inter` directory. For example:
///
///```rust
///#[example(file="my_file.rs")]
///```
///
/// This is equivalent to:
///
///```rust
///#[example(mod(file="my_file.rs"))]
///```
///
/// The generated example code file will be located at 
/// `examples/inter/my_file.rs`.
///
/// This option provides developers with an easy way to 
/// view and analyze the generated code, facilitating code 
/// inspection and potential code reuse.
///
/// ## main 
///
/// This option is used when specifying the `main` argument 
/// in the `example` macro. It generates two files within 
/// the `examples/inter` directory: the expanded code file 
/// and an additional `main.rs` file. 
///
///```rust
///#[example(main(file="my_file.rs"))]
///```
///
/// This option is particularly useful for testing and 
/// experimentation. It allows developers to quickly 
/// run and interact with the generated code by executing:
///
///```terminal
///$ cargo run --example inter
///```
///
/// The expanded code file will be located at 
/// `examples/inter/my_file.rs`, while the `main.rs` file 
/// serves as an entry point for running the example.
/// 
/// ## Configuration Options  
///```text 
/// 
///#[interthread::example( 
///   
///    mod ✔
///    main 
///
///    (   
///        file = "path/to/file.rs" ❗️ 
///
///        expand(actor,group) ✔
///    )
/// )]
/// 
/// 
/// default:    ✔
/// required:   ❗️
/// 
/// 
///```
/// 
/// # Arguments
/// 
/// - [`file`](#file)
/// - [`expand`](#expand) (default)
/// 
/// # file
/// 
/// 
/// The file argument is a required parameter of the [`example`](./attr.example.html) macro.
/// It expects the path to the file that needs to be expanded.
/// 
/// This argument is essential as it specifies the target file 
/// for code expansion.
/// 
/// One more time [`example`](./attr.example.html) macro can be 
/// placed on any item in any file within your `src` directory.
/// 
///  
/// # expand
/// 
/// This argument allows the user to specify which 
/// `interthread` macros to expand. 
/// 
/// By default, the value of `expand` includes 
/// the [`actor`](./attr.actor.html) and 
/// [`group`](./attr.group.html) macros.
/// 
/// For example, if you want to expand only the
/// [`actor`](./attr.actor.html) macro in the generated 
/// example code, you can use the following attribute:
/// 
/// ```rust
/// #[example(file="my_file.rs",expand(actor))]
/// ```
/// This will generate an example code file that includes 
/// the expanded code of the [`actor`](./attr.actor.html) macro,
/// while excluding other macros like 
/// [`group`](./attr.group.html).
/// 
 

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn example( attr: proc_macro::TokenStream, _item: proc_macro::TokenStream ) -> proc_macro::TokenStream {

    let mut eaa   = attribute::ExampleAttributeArguments::default();

    let aaa_parser = 
    syn::meta::parser(|meta| eaa.parse(meta));
    syn::parse_macro_input!(attr with aaa_parser);


    let (file, lib)  = file::expand_macros(&eaa.get_file(),&eaa.expand);

    let path = if eaa.main { 
        show::example_show(file, &eaa.get_file(), Some(lib))
    } else {
        show::example_show(file, &eaa.get_file(), None ) 
    };

    let msg = format!("The file has been SUCCESSFULLY created at {}",path.to_string_lossy());
    let note  = "To avoid potential issues and improve maintainability, it is recommended to comment out the macro after its successful execution. To proceed, please comment out the macro and re-run the compilation.";
    
    proc_macro_error::abort!( proc_macro2::Span::call_site(),msg; note = note);
    
}

 
/// ## Evolves a regular object into an actor
/// 
/// The macro is placed upon an implement block of an object
///  (`struct` or `enum`),
/// which has a public method named `new` returning  `Self`.
///
/// In case if the initialization could potentially fail, 
/// the method can be named `try_new` 
/// and return `Option<Self>` or `Result<Self>`.
/// 
/// The macro will copy method signatures from all 
/// public methods that do not consume the receiver, excluding 
/// methods like `pub fn foo(self, val: u8) -> ()` where `self` 
/// is consumed. Please ensure that the 
/// receiver is defined as `&self` or `&mut self`. 
/// 
/// If only a subset of methods is required to be 
/// accessible across threads, split the `impl` block 
/// into two parts. By applying the macro to a specific block, 
/// the macro will only consider the methods within that block.
/// 
/// ## Configuration Options
///```text 
/// 
/// #[interthread::actor( 
///   
///     channel = "inter"    ✔
///          0 || "unbounded" 
///               8 
/// 
///     lib     = "std"      ✔
///               "smol"
///               "tokio"
///               "async_std"
///     
///     edit    (            ✘
///               script
///               direct
///               play
///               live
///               live::new 
///              )           
///      
///     name    = ""         ✘
/// 
///     assoc   = false      ✔
/// 
///        id   = false      ✔
///  
/// )]
/// 
/// default:    ✔
/// no default: ✘
///
///```
///  
/// # Arguments
///  
///
/// - [`channel`](#channel)
/// - [`lib`](#lib) 
/// - [`edit`](#edit)
/// - [`name`](#name)
/// - [`assoc`](#assoc)
/// - [`id`](#id)
///
/// 
/// 
/// # channel
///
/// The `channel` argument specifies the type of channel. 
///
/// - `"inter"` (default)  
/// - `"unbounded"` or `0` 
/// - `8` ( [`usize`] buffer size)
/// > **Note:** The default `"inter"` option is experimental 
/// and primarily intended for experimentation purposes, 
/// specifically with the `lib = "std"` setting. 
/// It is recommended to avoid using this option 
/// unless you need it.
/// 
/// The two macros
/// ```rust
/// #[actor(channel="unbounded")]
/// ```
/// and
/// ```rust
/// #[actor(channel=0)]
/// ```
/// are in fact identical, both specifying same unbounded channel.
/// 
/// When specifying an [`usize`] value for the `channel` argument 
/// in the [`actor`](./attr.actor.html) macro, such as 
/// ```rust
/// #[actor(channel=4)]
/// ```
/// the actor will use a bounded channel with a buffer size of 4.
/// This means that the channel can hold up to 4 messages in its 
/// buffer before blocking/suspending the sender.
///
/// Using a bounded channel with a specific buffer size allows 
/// for control over the memory usage and backpressure behavior 
/// of the model. When the buffer is full, any further attempts 
/// to send messages will block/suspend until there is available space. 
/// This provides a natural form of backpressure, allowing the 
/// sender to slow down or pause message production when the 
/// buffer is near capacity.
/// 
/// # lib
///
/// The `lib` argument specifies the 'async' library to use.
///
/// - `"std"` (default)
/// - `"smol"`
/// - `"tokio"`
/// - `"async_std"`
///
///## Examples
///```rust
///use interthread::actor;
///
///struct MyActor;
///
///#[actor(channel=10, lib ="tokio")]
///impl MyActor{
///    pub fn new() -> Self{Self}
///}
///#[tokio::main]
///async fn main(){
///    let my_act = MyActorLive::new();
///}
///```
/// 
/// 
/// 
/// # edit
///
/// The `edit` argument specifies the available editing options.
/// When using this argument, the macro expansion will 
/// **exclude** the code related to `edit` options, 
/// allowing the user to manually implement and 
/// customize those parts according to their specific needs.
/// 
/// - [`script`](index.html#script)
/// - [`direct`](index.html#direct)
/// - [`play`](index.html#play)
/// - [`live`](index.html#live)
/// - `live::new`  
///
/// 
/// ## Examples
///```rust
///
///use std::sync::mpsc;
///use interthread::actor;
/// 
///pub struct MyActor {
///    value: i8,
///}
/// // we will edit `play` function
/// #[actor(channel=2, edit(play))]
///impl MyActor {
///
///    pub fn new( value: i8 ) -> Self {
///        Self{value}
///    }
///    pub fn increment(&mut self) -> i8{
///        self.value += 1;
///        self.value
///    }
///}
///
///// manually create "play" function 
///// use `example` macro to copy paste
///// `play`'s body 
///pub fn my_actor_play( 
///     receiver: mpsc::Receiver<MyActorScript>,
///    mut actor: MyActor) {
///    // set a custom variable 
///    let mut call_counter = 0;
///    while let Ok(msg) = receiver.recv() {
///        // do something 
///        // like 
///        println!("Value of call_counter = {}",call_counter);
///
///        // `direct` as usual 
///        msg.my_actor_direct(&mut actor);
///
///        // increment the counter as well
///        call_counter += 1;
///    }
///    eprintln!(" the end ");
///}
///
///
///fn main() {
///
///    let my_act       = MyActorLive::new(0);
///    let mut act_a = my_act.clone();
///    
///
///    let handle_a = std::thread::spawn(move || -> i8{
///        act_a.increment()
///    });
///
///    let value = handle_a.join().unwrap();
///    
///    assert_eq!(value, 1);
///
///    // and it will print the value of 
///    // call_counter
///}
///```
///
/// > **Note:** The expanded `actor` can be viewed using [`example`](./attr.example.html) macro. 
/// 
/// 
/// Now, let's explore a scenario where we want to manipulate or 
/// even return a type from the [`play`](index.html#play) 
/// component by invoking a method on the [`live`](index.html#live) 
/// component. We can easily modify the generated code to 
/// enable this functionality.
/// 
/// ## Examples
///```rust
///use std::sync::mpsc;
///use interthread::actor;
/// 
///pub struct MyActor {
///    value: i8,
///}
/// #[actor(channel=2, edit(play))]
///impl MyActor {
///
///    pub fn new( value: i8 ) -> Self {
///        Self{value}
///    }
///    pub fn increment(&mut self) -> i8{
///        self.value += 1;
///        self.value
///    }
///    // it's safe to hack the macro in this way
///    // having `&self` as receiver along  with
///    // other things creates a `Script` variant  
///    // We'll catch it in `play` function
///    pub fn play_get_counter(&self)-> Option<u32>{
///        None
///    }
///
///}
///
///// manually create "play" function 
///// use `example` macro to copy paste
///// `play`'s body
///pub fn my_actor_play( 
///     receiver: mpsc::Receiver<MyActorScript>,
///    mut actor: MyActor) {
///    // set a custom variable 
///    let mut call_counter = 0;
///
///    while let Ok(msg) = receiver.recv() {
///
///        // match incoming msgs
///        // for `play_get_counter` variant
///        match msg {
///            // you don't have to remember the 
///            // the name of the `Script` variant 
///            // your text editor does it for you
///            // so just choose the variant
///            MyActorScript::PlayGetCounter { output  } =>
///            { let _ = output.send(Some(call_counter));},
///            
///            // else as usual 
///            _ => { msg.my_actor_direct(&mut actor); }
///        }
///        call_counter += 1;
///    }
///    eprintln!("the end");
///}
///
///
///fn main() {
///
///    let my_act = MyActorLive::new(0);
///    let mut act_a = my_act.clone();
///    let mut act_b = my_act.clone();
///
///    let handle_a = std::thread::spawn(move || {
///        act_a.increment();
///    });
///    let handle_b = std::thread::spawn(move || {
///        act_b.increment();
///    });
///    
///    let _ = handle_a.join();
///    let _ = handle_b.join();
///
///
///    let handle_c = std::thread::spawn(move || {
///
///        // as usual we invoke a method on `live` instance
///        // which has the same name as on the Actor object
///        // but 
///        if let Some(counter) = my_act.play_get_counter(){
///
///            println!("This call never riched the `Actor`, 
///            it returns the value of total calls from the 
///            `play` function ,call_counter = {:?}",counter);
///
///            assert_eq!(counter, 2);
///        }
///    });
///    let _ = handle_c.join();
///
///}
///```
/// Let's take a moment to rearrange our example. 
/// 
/// 
/// ## Examples
///```rust
///use std::sync::mpsc;
///use interthread::actor;
/// 
///pub struct MyActor {
///    value: i8,
///}
/// #[actor(channel=2, edit(play))]
///impl MyActor {
///
///    pub fn new( value: i8 ) -> Self {
///        Self{value}
///    }
///    pub fn increment(&mut self) -> i8{
///        self.value += 1;
///        self.value
///    }
///    pub fn play_get_counter(&self)-> Option<u32>{
///        None
///    }
///
///}
///
///
///// incapsulate the matching block
///// inside `Script` impl block
///// where the `direct`ing is happening
///// to keep our `play` function nice
///// and tidy 
///impl MyActorScript {
///    pub fn custom_direct(self,
///           actor: &mut MyActor, 
///           counter: &u32 ){
///
///        // the same mathing block 
///        // as in above example    
///        match self {
///            MyActorScript::PlayGetCounter { output  } =>
///            { let _ = output.send(Some(counter.clone()));},
///            
///            // else as usual 
///            msg => { msg.my_actor_direct(actor); }
///        }
///    } 
///}
///
///// manually create "play" function 
///// use `example` macro to copy paste
///// `play`'s body
///pub fn my_actor_play( 
///     receiver: mpsc::Receiver<MyActorScript>,
///    mut actor: MyActor) {
///    // set a custom variable 
///    let mut call_counter = 0;
///    
///    // nice and tidy while loop ready
///    // for more wild things to happen
///    while let Ok(msg) = receiver.recv() {
///        
///        // this is the invocation
///        // of MyActorScript.custom_direct()
///        msg.custom_direct(&mut actor, &call_counter);
///
///        call_counter += 1;
///    }
///    eprintln!("the end");
///}
///
///
///fn main() {
///
///    let my_act = MyActorLive::new(0);
///    let mut act_a = my_act.clone();
///    let mut act_b = my_act.clone();
///
///    let handle_a = std::thread::spawn(move || {
///        act_a.increment();
///    });
///    let handle_b = std::thread::spawn(move || {
///        act_b.increment();
///    });
///    
///    let _ = handle_a.join();
///    let _ = handle_b.join();
///
///
///    let handle_c = std::thread::spawn(move || {
///
///        if let Some(counter) = my_act.play_get_counter(){
///
///            println!("This call never riched the `Actor`, 
///            it returns the value of total calls from the 
///            `play` function ,call_counter = {:?}",counter);
///
///            assert_eq!(counter, 2);
///        }
///    });
///    let _ = handle_c.join();
///
///}
///```
/// 
/// # name
/// 
/// The `name` attribute allows developers to provide a 
/// custom name for `actor`, overriding the default 
/// naming conventions of the crate. This can be useful 
/// when there are naming conflicts or when a specific 
/// naming scheme is desired.  
/// 
/// - "" (default): No name specified
///
/// ## Examples
///```rust
///use interthread::actor;
/// 
///pub struct MyActor;
/// 
///#[actor(name="OtherActor")]
///impl MyActor {
///
///   pub fn new() -> Self {Self}
///}
///fn main () {
///   let other_act = OtherActorLive::new();
///}
///```
/// 
/// 
/// 
/// # assoc
/// 
/// The `assoc` option indicates whether **associated**  **functions**
/// ( also known as static methods ) that **return** a type of the actor struct are included 
/// in generated code as instance methods, allowing them to be invoked on 
/// the generated struct itself. 
///
/// - true  
/// - false (default)
/// 
///  ## Examples
///```rust
///use interthread::actor;
///pub struct Aa;
///  
/// 
///#[actor(name="Bb", assoc=true)]
///impl Aa {
///
///    pub fn new() -> Self { Self{} }
/// 
///    // we don't have a `&self`
///    // receiver 
///    pub fn is_even( n: u8 ) -> bool {
///        n % 2 == 0
///    }
///}
///
///fn main() {
///    
///    let bb = BbLive::new();
/// 
///    // but we can call it 
///    // as if there was one   
///    assert_eq!(bb.is_even(84), Aa::is_even(84));
///}
///
///```
/// # id
/// 
/// If this argument is set to `true`, the following 
/// additions and implementations are generated :
/// 
/// Within the [`live`](index.html#live) struct definition, the following
/// fields are generated:
/// 
/// - `pub debut: std::time::SystemTime`
/// - `pub name: String`
/// 
/// The following traits are implemented for the [`live`](index.html#live) struct:
/// 
/// - `PartialEq`
/// - `PartialOrd`
/// - `Eq`
/// - `Ord`
/// 
/// These traits allow for equality and ordering 
/// comparisons based on the `debut`value.
/// The `name` field is provided for user needs only and is not 
/// taken into account when performing comparisons. 
/// It serves as a descriptive attribute or label 
/// associated with each instance of the live struct.
/// 
/// In the [`script`](index.html#script) struct implementation block, which 
/// typically encapsulates the functionality of the model,
/// a static method named `debut` is generated. This 
/// method returns the current system time and is commonly 
/// used to set the `debut` field when initializing 
/// instances of the [`live`](index.html#live) struct.
///
/// Use macro [`example`](./attr.example.html) to see the generated code.
/// 
/// 
/// ## Examples
///  
///```rust
///use std::thread::spawn;
///pub struct MyActor ;
///
///#[interthread::actor(channel=2, id=true)] 
///impl MyActor {
///    pub fn new() -> Self { Self{} } 
///}
///fn main() {
///
///    let actor_1 = MyActorLive::new();
///
///    let handle_2 = spawn( move || { 
///        MyActorLive::new()
///    });
///    let actor_2 = handle_2.join().unwrap();
///
///    let handle_3 = spawn( move || {
///        MyActorLive::new()
///    });
///    let actor_3 = handle_3.join().unwrap();
///    
///    // they are the same type objects
///    // but serving differrent threads
///    // different actors !   
///    assert!(actor_1 != actor_2);
///    assert!(actor_2 != actor_3);
///    assert!(actor_3 != actor_1);
///
///    // sice we know the order of invocation
///    // we correctly presume
///    assert_eq!(actor_1 > actor_2, true );
///    assert_eq!(actor_2 > actor_3, true );
///    assert_eq!(actor_3 < actor_1, true );
///
///    // but if we check the order by `debute` value
///    assert_eq!(actor_1.debut < actor_2.debut, true );
///    assert_eq!(actor_2.debut < actor_3.debut, true );
///    assert_eq!(actor_3.debut > actor_1.debut, true );
///    
///    // This is because the 'debut' 
///    // is a time record of initiation
///    // Charles S Chaplin (1889)
///    // Keanu Reeves      (1964)
///
///
///    // we can count `live` instances for 
///    // every model
///    use std::sync::Arc;
///    let mut a11 = actor_1.clone();
///    let mut a12 = actor_1.clone();
///
///    let mut a31 = actor_3.clone();
///
///    assert_eq!(Arc::strong_count(&actor_1.debut), 3 );
///    assert_eq!(Arc::strong_count(&actor_2.debut), 1 );
///    assert_eq!(Arc::strong_count(&actor_3.debut), 2 );
///            
///
///    // the name field is not taken 
///    // into account when comparison is
///    // perfomed       
///    assert!( a11 == a12);
///    assert!( a11 != a31);
///
///    a11.name = String::from("Alice");
///    a12.name = String::from("Bob");
///
///    a31.name = String::from("Alice");
///
///    assert_eq!(a11 == a12, true );
///    assert_eq!(a11 != a31, true );
///
///}
///``` 
/// 
/// 

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn actor( attr: proc_macro::TokenStream, item: proc_macro::TokenStream ) -> proc_macro::TokenStream {
    
    let impl_block                      = syn::parse_macro_input!(item as syn::ItemImpl);
    let mut paaa    = attribute::ParseActorAttributeArguments::default();

    let attr_str = attr.clone().to_string();

    if !attr_str.is_empty(){

        let aaa_parser  = 
        syn::meta::parser(|meta| paaa.parse(meta));
        syn::parse_macro_input!(attr with aaa_parser);
    }
    let aaa = paaa.get_arguments();

    check::channels_import( &aaa.lib );
    
    let mut inter_gen_actor = actor_gen::ActorMacroGeneration::new( /*name,*/ aaa, impl_block );
    let code = inter_gen_actor.generate();
    quote::quote!{#code}.into()
   
}

/// ## Currently under development (((
/// 
/// The `group` macro, although not currently included 
/// in the `interthread` crate.It aims to address 
/// several critical challenges encountered when
///  working with the `actor` macro:
/// 
/// - Instead of creating separate threads for each object, 
/// the `group` macro will enable the user to create an actor 
/// that represents a group of objects, consolidating 
/// their processing and execution within a single thread.
/// 
/// 
/// - In scenarios where objects are already created or imported,
/// and the user does not have the authority to implement 
/// additional methods such as  "new" or "try_new",
/// the `group` macro should offer a way to include 
/// these objects as part of the actor system.
///
/// Although the `group` macro is not currently part of the 
/// `interthread` crate, its development aims to offer a 
/// comprehensive solution to these challenges, empowering 
/// users to efficiently manage groups of objects within an 
/// actor system.
/// 
/// Check `interthread` on ['GitHub'](https://github.com/NimonSour/interthread.git)
/// 

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn group( _attr: proc_macro::TokenStream, _item: proc_macro::TokenStream ) -> proc_macro::TokenStream {
    let msg = "The \"group\" macro is currently under development and is not yet implemented in the `interthread` crate.";
    proc_macro_error::abort!( proc_macro2::Span::call_site(),msg );
}






