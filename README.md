
# interthread

> "The basic idea behind an actor is to spawn a 
self-contained task that performs some job independently
of other parts of the program. Typically these actors
communicate with the rest of the program through 
the use of message passing channels. Since each actor 
runs independently, programs designed using them are 
naturally parallel."
> - Alice Ryhl 


For a comprehensive understanding of the underlying
concepts and implementation details of the Actor Model,  
it's recommended to read the article  [Actors with Tokio](https:/ryhl.io/blog/actors-with-tokio/)
 by Alice Ryhl ( also known as _Darksonn_ ). 
This article not only inspired the development of the 
`interthread` crate but also serves as foundation 
for the Actor Model implementation logic in it. 


## What is the problem ?

To achieve parallel execution of individual objects 
within the same program, it is challenging due 
to the need for various types that are capable of 
working across threads. The main difficulty 
lies in the fact that as you introduce thread-related types,
you can quickly lose sight of the main program 
idea as the focus shifts to managing thread-related 
concerns.
It involves using constructs like threads, locks, channels,
and other synchronization primitives. These additional 
types and mechanisms introduce complexity and can obscure 
the core logic of the program.


Moreover, existing libraries like [`actix`](https://docs.rs/actixlatest/actix/), [`axiom`](https://docs.rs/axiom/latest/axiom/), 
designed to simplify working within the Actor Model,
often employ specific concepts, vocabulary, traits and types that may
be unfamiliar to users who are less experienced with 
asynchronous programming and futures. 

## Solution 
 
The [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html) macro -  when applied to the 
implementation block of a given "MyActor" object,
generates additional types and functions 
that enable communication between threads.

A notable outcome of applying this macro is the 
creation of the `MyActorLive` struct ("ActorName" + "Live"),
which acts as an interface/handle to the `MyActor` object.
`MyActorLive` retains the exact same public method signatures
as `MyActor`, allowing users to interact with the actor as if 
they were directly working with the original object.

### Examples


Filename: Cargo.toml

```text
interthread = "0.1.1"
oneshot     = "0.1.5" 
```

Filename: main.rs
```rust

pub struct MyActor {
    value: i8,
}

#[interthread::actor(channel=2)] // <-  this is it 
impl MyActor {

    pub fn new( v: i8 ) -> Self {
       Self { value: v } 
    }
    pub fn increment(&mut self) {
        self.value += 1;
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
}
 
fn main() {

    let actor = MyActorLive::new(5);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = std::thread::spawn( move || { 
    actor_a.increment();
    });

    let handle_b = std::thread::spawn( move || {
    actor_b.add_number(5)
    });

    let _  = handle_a.join();
    let hb = handle_b.join().unwrap();

    // we never know which thread will
    // be first to call the actor so
    // hb = 10 or 11
    assert!(hb >= 10);

    assert_eq!(actor.get_value(), 11);
}

```
 
An essential point to highlight is that when invoking 
`MyActorLive::new`, not only does it return an instance 
of `MyActorLive`, but it also spawns a new thread that 
contains an instance of `MyActor` in it. 
This introduces parallelism to the program.

The code generated by [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html) takes 
care of the underlying message routing and synchronization, 
allowing developers to rapidly prototype their application's
core functionality. This fast sketching capability is
particularly useful when exploring different design options, 
experimenting with concurrency models, or implementing 
proof-of-concept systems.



The same example can be run in 
[tokio](https://crates.io/crates/tokio),
[async-std](https://crates.io/cratesasync-std), 
and [smol](https://crates.io/cratessmol), 
with the only difference being that the methods will 
be marked as `async` and need to be `await`ed for 
asynchronous execution."

Filename: Cargo.toml

```text
interthread = "0.1.1"
tokio       = { version="1.28.2",features=["full"]}
```
Filename: main.rs

```rust

pub struct MyActor {
    value: i8,
}

#[interthread::actor(channel=2,lib="tokio")] // <-  one line )
impl MyActor {

    pub fn new( v: i8 ) -> Self {
       Self { value: v } 
    }
    // if the "lib" is defined
    // object methods can be "async" 
    pub async fn increment(&mut self) {
        self.value += 1;
    }
    pub fn add_number(&mut self, num: i8) -> i8 {
        self.value += num;
        self.value
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
}

#[tokio::main]
async fn main() {

    let actor = MyActorLive::new(5);

    let mut actor_a = actor.clone();
    let mut actor_b = actor.clone();

    let handle_a = tokio::spawn( async move { 
    actor_a.increment().await;
    });

    let handle_b = tokio::spawn( async move {
    actor_b.add_number(5).await
    });

    let _  = handle_a.await;
    let hb = handle_b.await.unwrap();

    // hb = 10 or 11
    assert!(hb >= 10);

    assert_eq!(actor.get_value().await, 11);
}
```
The crate also includes a powerful macro called [`example`](https://docs.rs/interthread/latest/interthread/attr.example.html) that can expand the [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html) macro, ensuring that users always have the opportunity to visualize and interact with the generated code. Which makes [`actor`](https://docs.rs/interthread/latest/interthread/attr.actor.html)  100%  transparent macro . 


For more details, check out `interthread` on 
[Docs.rs](https://docs.rs/interthread#sdpl).

You've paid for 5 cores, don't code on one.

