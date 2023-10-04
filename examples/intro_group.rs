


pub struct AnyOtherType;


pub struct Aa(pub u8);
pub struct Bb(pub u8);
pub struct Cc(pub u8);


pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    pub c: Cc,
    any: AnyOtherType,
}

/*
For a struct

pub struct AaBbCc {
    pub a: Aa,
    pub b: Bb, 
    c: Cc,
}

available `edit` arguments are:

#[interthread::group(
    file="path/to/abc.rs",
    edit(
        file   <-  !!!

        script( def, imp(..), trt(..) ),
        live(   def, imp(..), trt(..) ),

        a( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         ),

        b( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         )
    )
)]

#[interthread::group(
    file="path/to/abc.rs",
    edit(
        file   

        script( def, imp(..), trt(..) ),
        live(   def, imp(..), trt(..) ),

        Self::a( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         ),

        Self::b( script( def, imp(..), trt(..) ),
           live(   def, imp(..), trt(..) ), 
         )
    )
)]


Note: the `file` ident inside the `edit` argument. 

#[interthread::group(
    file="path/to/abc.rs",
    edit 
)]

The above `edit` argument triggers the whole model to be written.


#[interthread::group(
    file = "path/to/abc.rs",
    path = ( a("path/to/a.rs"), b("path/to/b.rs") ),

)]

#[interthread::actor(
    file="path/to/abc.rs",
    edit( file(
            script( def, imp(..), trt(..) ),
            live(   def, imp(..), trt(..) ),
        )
    )
)]



//////////////

1) About 'file' argument.
    Actor ) 'file' argument migrates into 'edit' argument.
           When the argument is defined it works as it should 
           editing (writing)to the file. If an additional 
           file-active argument is defined anywere in the scope of 
           'edit' list, than to the file will be written just 
           arguments defined in 'file-active' scope.

    Group ) File argument is defined outside of 'edit' argument.
            To include it inside the 'edit' argument to enforce rules of 
            'file-active' just include a 'file-active' (`file`) where normally
            will use a file key value inside the  'Actor' 'edit' argument.


2) Examples of usage : 
    
    a) write all 

        actor)

        group) 

Examples:


#[interthread::group(
    file="path/to/abc.rs",
    edit(
        file ,  

        script( def, imp, trt ),
        file(live(   def, imp, trt)),

        Self::a( script( def, imp(file(bla)), trt ),
           live(   file(def), imp, trt ), 
         ),

        Self::b( file(script( def, imp)),
           live(   def, imp, trt ), 
         )
    )
)]


#[interthread::group(
    file="path/to/abc.rs",
    edit(
        script( def, imp, trt ),
        live(   def, imp, trt),

        Self::a( script( def, imp(bla), trt ),
                   live( def, imp, trt ), 
         ),

        Self::b( script( def, imp),
                   live( def, imp, trt ), 
         )
    )
)]








*/




// impl AaBbCc {

//     pub fn new(a:u8, b:u8, c:u8) -> Self {
//         Self { a: Aa(a), b: Bb(b), c: Cc(c) }
//     }

// }

pub struct MyActor {
    value: i8,
}
// #[interthread::actor(edit(live(imp(get_value))))]  V

// #[interthread::actor(edit(live(imp(increment))))]   
// #[interthread::actor(file = "examples/intro_group.rs", edit(live(def, imp),script(def(file))))]  
// #[interthread ::
// actor(file = "examples/intro_group.rs", edit(live(def, imp), script(def)))] 
// #[interthread :: actor(file = "examples/intro_group.rs", edit(script))]


impl MyActor {
    pub fn new(v: i8) -> Self {
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



// #[interthread::example(path = "examples/intro_group.rs")]
pub fn main(){

}