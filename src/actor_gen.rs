use crate::attribute::{ActorAttributeArguments,AALib,AAChannel,AAExpand};
use crate::name;
use crate::method;
use crate::error;
use crate::generics;

use proc_macro_error::abort;
use std::boxed::Box;
use syn::{Ident,Signature,Item,Type,Visibility };
use quote::{quote,format_ident};
use proc_macro2::TokenStream;

pub fn live_static_method( 
    actor_name: &Ident,
         ident: Ident, 
           vis: Visibility,
       mut sig: Signature,
          args: TokenStream,
     live_mets: &mut Vec<(Ident,TokenStream)> ) {

    method::change_signature_refer(&mut sig);
    let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
    let stat_met = quote! {
        #vis #sig {
            #actor_name::#ident #args #await_call
        }
    };
    live_mets.push((ident,stat_met));
}


// returns  (code,edit) TokenStreams 
pub fn actor_macro_generate_code( aaa: ActorAttributeArguments, item: Item, mac: &AAExpand ) -> (TokenStream, TokenStream){
    
    let mut script_def;
    let mut script_mets = vec![];
    let mut script_trts = vec![];
  
    let mut live_def;
    let mut live_mets = vec![];
    let mut live_trts = vec![];


    let mut script_fields   = vec![];
    let mut direct_arms     = vec![];
    let mut debug_arms      = vec![];


    let (actor_name,
        actor_type,
        generics) = name::get_name_and_type(mac,&item,);
    
    let ( impl_generics,
            ty_generics,
           where_clause ) = generics::get_parts(&generics);

    let (actor_methods, 
         met_new) =
         method::get_methods( &actor_type,item.clone(),aaa.assoc );

    let met_new = if met_new.is_none() {
        if method::is_trait(&actor_type) {
            let (msg,note) = error::trait_new_sig(&actor_type,false);
            abort!(item,msg;note=note);
        } else {
            let msg = format!("Can not find public/restricted  method `new` or `try_new` for {:?} object.",actor_name.to_string());
            let (note,help) = error::met_new_note_help(&actor_name);
            abort!(item,msg;note=note;help=help);
        }
    } else { met_new.unwrap() };
    
    // Giving a new name if specified 
    let cust_name   = if aaa.name.is_some(){ aaa.name.clone().unwrap() } else { actor_name.clone() }; 
    
    let script_name = &name::script(&cust_name);
    let live_name   = &name::live(&cust_name);
    

    let (live_field_sender,
        play_input_receiver, 
        new_live_send_recv , 
        live_meth_send_recv, 
        script_field_output, 
        live_send_input,
        live_recv_output ) = channels( &aaa.lib, &aaa.channel, &cust_name, &ty_generics);


    
    let direct_async_decl = 
    if actor_methods.iter().any(|x| x.is_async()) { 
        Some(quote!{async})
    } else { None };

    let play_async_decl   = 

        match &aaa.lib {
            AALib::Std => {
                if direct_async_decl.is_some(){ 
                    let pos = actor_methods.iter().position(|x| x.is_async()).unwrap();
                    error::abort_async_no_lib(&actor_name,&actor_methods[pos]);
                } 
                None
            },
            _ => { Some(quote!{async}) },
        };

    let new_vis = met_new.vis.clone();

    for method in actor_methods.clone() {
        
        let (mut sig, script_field_name) = method.get_sig_and_field_name();

        let await_call = sig.asyncness.as_ref().map(|_|quote!{.await});
        method::to_async(&aaa.lib, &mut sig);

        let error_send = error::direct_send(&script_name,&script_field_name);

        // Debug arm
        let add_arm = | debug_arms: &mut Vec<TokenStream>,ident: &Ident | {

            let str_field_name = ident.to_string();

            let debug_arm = quote! {
                #script_name :: #script_field_name {..} => write!(f, #str_field_name),
            };
            debug_arms.push(debug_arm);
        };

        match method {

            method::ActorMethod::Io   { vis, ident, stat,  arguments, output,.. } => {
                let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
                }
                else {
                    // Debug Arm push
                    add_arm(&mut debug_arms, &script_field_name);

                    // Direct Arm
                    let arm_match        = quote! { 
                        #script_field_name { input: #args_ident,  output: send }
                    };
                    let direct_arm       = quote! {
                        #script_name :: #arm_match => {send.send( actor.#ident #args_ident #await_call ) #error_send ;}
                    };
                    direct_arms.push(direct_arm);
                    
                    // Live Method
                    let live_met      = quote! {

                        #vis #sig {
                            #live_meth_send_recv
                            let msg = #script_name :: #arm_match;
                            #live_send_input
                            #live_recv_output
                        }
                    };

                    live_mets.push((ident,live_met));

                    // Script Field Struct
                    let output_type      = (&*script_field_output)(output);

                    let script_field = quote! {
                        #script_field_name {
                            input: #args_type,
                            #output_type
                        }
                    };

                    script_fields.push(script_field);
                }
            },
            method::ActorMethod::I    { vis, ident, arguments ,..} => {
                
                let (args_ident,args_type) = method::arguments_ident_type(&arguments);
                
                // Debug Arm push
                add_arm(&mut debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name{ input: #args_ident }
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {actor.#ident #args_ident #await_call;},
                };
                direct_arms.push(direct_arm);

                // Live Method
                let live_met = quote!{
    
                    #vis #sig {
                        let msg = #script_name::#arm_match ;
                        #live_send_input
                    }
                };
                live_mets.push((ident,live_met));
            


                // Script Field Struct
                let script_field = quote!{
                    #script_field_name {
                        input: #args_type,
                    }
                };
                script_fields.push(script_field);

            },
            method::ActorMethod::O    { vis, ident, stat, output ,..} => {
                let (args_ident,_) = method::arguments_ident_type(&vec![]);

                if stat {
                    live_static_method(&actor_name,ident, vis, sig, args_ident,&mut live_mets)
                }
                else {
                    
                    // Debug Arm push
                    add_arm(&mut debug_arms, &script_field_name);

                    // Direct Arm
                    let arm_match = quote!{ 
                        #script_field_name{  output: send }
                    };
        
                    let direct_arm = quote!{
                        #script_name::#arm_match => {send.send(actor.#ident #args_ident #await_call) #error_send ;}
                    };
                    direct_arms.push(direct_arm);



                    // Live Method
                    let live_met = quote!{
                    
                        #vis #sig {
                            #live_meth_send_recv
                            let msg = #script_name::#arm_match ;
                            #live_send_input
                            #live_recv_output
                        }
                    };
                    live_mets.push((ident, live_met));
                
                    // Script Field Struct
                    let output_type  = (&*script_field_output)(output);

                    let script_field = quote!{
                        #script_field_name {
                            #output_type
                        }
                    };
                    script_fields.push(script_field);
                }
            },
            method::ActorMethod::None { vis, ident ,..} => {

                // Debug Arm push
                add_arm(&mut debug_arms, &script_field_name);

                // Direct Arm
                let arm_match = quote!{ 
                    #script_field_name {} 
                };
    
                let direct_arm = quote!{
                    #script_name::#arm_match => {actor.#ident () #await_call;},
                };
                direct_arms.push(direct_arm);

                // Live Method
                let live_met = quote!{
                
                    #vis #sig {
                        let msg = #script_name::#arm_match ;
                        #live_send_input
                    }
                };
                live_mets.push((ident,live_met));
            
                // Script Field Struct
                let script_field = quote!{
                    
                    #script_field_name {}
                };
                script_fields.push(script_field);
            },
        }
    } 


    // METHOD NEW
    { 
        let new_sig             = &met_new.new_sig;
        let func_new_name           = &new_sig.ident;
        let (args_ident, _ )   = method::arguments_ident_type(&met_new.get_arguments());
        let live_var                 = format_ident!("actor_live");
        let unwrapped          = met_new.unwrap_sign();
        let return_statement   = met_new.live_ret_statement(&live_var);
        let vis                = &met_new.vis.clone();

        let live_new_spawn = |play_args:TokenStream| {
            match aaa.lib {
                AALib::Std      => {
                    quote!{ std::thread::spawn(|| { #script_name :: play(#play_args) } );}
                },
                AALib::Smol     => {
                    quote!{ smol::spawn( #script_name :: play(#play_args) ).detach();} 
                },
                AALib::Tokio    => {
                    quote!{ tokio::spawn( #script_name :: play(#play_args) );}
                },
                AALib::AsyncStd => {
                    quote!{ async_std::task::spawn( #script_name :: play(#play_args) );}
                },
            }
        };

        let (init_actor, play_args) = {
            let id_debut_name = if aaa.id {quote!{ ,debut,name}} else {quote!{}};
            match  aaa.channel {
                AAChannel::Inter => {
                    ( quote!{ Self{ queue: queue.clone(), condvar: condvar.clone() #id_debut_name } }, quote!{ queue, condvar, actor  } )
                },
                _  => {
                    ( quote!{ Self{ sender #id_debut_name } }, quote!{ receiver, actor } )
                },
            }
        };

        let spawn = live_new_spawn(play_args);
        let turbofish = ty_generics.as_ref().map(|x| x.as_turbofish());
        let (id_debut,id_name)  =  
        if aaa.id {
            (quote!{let debut =  #script_name #turbofish ::debut();},
                quote!{let name  = String::from("");})
        } else { (quote!{}, quote!{}) };
        
        let func_new_body = quote!{

            #vis #new_sig {
                let actor = #actor_name:: #func_new_name #args_ident #unwrapped;
                #new_live_send_recv
                #id_debut
                #id_name
                let #live_var = #init_actor;
                #spawn
                #return_statement
            }
        };
        live_mets.insert(0,(new_sig.ident.clone(),func_new_body));
    };

    // LIVE INTER METHODS AND TRAITS
    if aaa.id {

        live_mets.push((format_ident!("inter_get_debut"),
        quote!{
            #new_vis fn inter_get_debut(&self) -> std::time::SystemTime {
                *self.debut
            }
        }));
        
        live_mets.push((format_ident!("inter_get_count"),
        quote!{
            #new_vis fn inter_get_count(&self) -> usize {
                std::sync::Arc::strong_count(&self.debut)
            }
        }));

        live_mets.push((format_ident!("inter_set_name"),
        quote!{
            #new_vis fn inter_set_name<Name: std::string::ToString>(&mut self, name: Name) {
                self.name = name.to_string();
            }
        }));


        live_mets.push((format_ident!("inter_get_name"),
        quote!{    
            #new_vis fn inter_get_name(&self) -> &str {
                &self.name
            } 
        }));

        script_mets.push((format_ident!("debut"),
        quote!{

            pub fn debut ()-> std::sync::Arc<std::time::SystemTime> {
                static LAST: std::sync::Mutex<std::time::SystemTime> = std::sync::Mutex::new(std::time::SystemTime::UNIX_EPOCH);
        
                let mut last_time = LAST.lock().unwrap();
                let mut next_time = std::time::SystemTime::now();
        
                // we check for 'drift'
                // as described in docs 
                while !(*last_time < next_time)  {
                    // in case if they are just equal
                    // add a nano but don't break the loop yet
                    if *last_time == next_time {
                        next_time += std::time::Duration::new(0, 1);
                    } else {
                        next_time = std::time::SystemTime::now();
                    }
                }
                // update LAST 
                *last_time = next_time.clone();
                std::sync::Arc::new(next_time)
            }
        }));
        
        live_trts.push((format_ident!("PartialEq"),
        quote!{
            impl #ty_generics PartialEq for #live_name #ty_generics #where_clause{
                fn eq(&self, other: &Self) -> bool {
                    *self.debut == *other.debut
                }
            }
        }));

        live_trts.push((format_ident!("Eq"),
        quote!{
            impl #ty_generics Eq for #live_name #ty_generics #where_clause {}
        }));  

        live_trts.push((format_ident!("PartialOrd"),
        quote!{
            impl #ty_generics PartialOrd for #live_name #ty_generics #where_clause{
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    other.debut.partial_cmp(&self.debut)
                }
            }
        }));   

        live_trts.push((format_ident!("Ord"),
        quote!{
            impl #ty_generics Ord for #live_name #ty_generics #where_clause {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    other.debut.cmp(&self.debut)
                }
            }
        }));  
    } 

    // SCRIPT DEFINITION
    script_def = {

        quote! {
            #new_vis enum #script_name #ty_generics #where_clause {
                #(#script_fields),*
            }
        }
    };

    // DIRECT
    {

        // let async_decl= async_token(direct_async);
        script_mets.push((format_ident!("direct"),
        quote!{
            #new_vis #direct_async_decl fn direct (self, actor: &mut #actor_type #ty_generics ) {
                match self {
                    #(#direct_arms)*
                }
            }
        }));
    }


    // PLAY
    {
        let await_call  = direct_async_decl.as_ref().map(|_| quote!{.await});
        let end_of_play = error::end_of_life(&actor_name); 

        // needs to be pushed into script_mets
        let play_method = match aaa.channel {

            AAChannel::Unbounded |
            AAChannel::Buffer(_) => {

                let ok_or_some = match aaa.lib {
                    AALib::Tokio => quote!{Some},
                    _ => quote!{Ok}
                };
                quote! {
                    #new_vis #play_async_decl fn play ( #play_input_receiver mut actor: #actor_type #ty_generics ) {
                        while let #ok_or_some (msg) = receiver.recv() #await_call {
                            msg.direct ( &mut actor ) #await_call;
                        }
                        #end_of_play
                    }
                }
            },

            AAChannel::Inter => {
                //impl drop for live while here
                live_trts.push((format_ident!("Drop"),
                quote!{
                    impl #ty_generics Drop for #live_name #ty_generics #where_clause {
                        fn drop(&mut self){
                            self.condvar.notify_one();
                        }
                    }
                }));

                let error_msg = error::play_guard(&actor_name);

                quote!{
                    #new_vis #play_async_decl fn play ( #play_input_receiver mut actor: #actor_type #ty_generics ) {

                        let queuing = || -> Option<Vec< #script_name #ty_generics>> {
                            let mut guard = queue.lock().expect(#error_msg);
                            while guard.as_ref().unwrap().is_empty() {
                                if std::sync::Arc::strong_count(&queue) > 1{
                                    guard = condvar.wait(guard).expect(#error_msg);
                                } else { return None }
                            }
                            let income = guard.take();
                            *guard = Some(vec![]);
                            income
                        };
                        while let Some(msgs)  = queuing(){
                            for msg in msgs {
                                msg.direct (&mut actor) #await_call;
                            }
                        }
                        #end_of_play
                    }
                }
            },
        };
        script_mets.push(( format_ident!("play"), play_method ));
    }
    
    // SCRIPT TRAIT (Debug)
    {   

        let str_script_name = script_name.to_string();
        let body = 
        if debug_arms.is_empty() { 
            quote!{ write!(f, #str_script_name )} 
        } else {
            quote!{ match self { #(#debug_arms)* } }
        };
        script_trts.push((format_ident!("Debug"),
        quote! {
            impl #ty_generics std::fmt::Debug for #script_name #ty_generics #where_clause {
        
                fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result {
                    #body
                }
            }
        }));
    }


    // LIVE DEFINITION
    live_def = {
        let (debut_field, name_field) = if aaa.id {
            ( quote!{ pub debut: std::sync::Arc<std::time::SystemTime>,},
            quote!{ pub name: String,} )
        } else { (quote!{}, quote!{})};   

        quote!{
            #[derive(Clone)]
            #new_vis struct #live_name #ty_generics #where_clause {
                #live_field_sender
                #debut_field
                #name_field
            }
        }
    };

    // Create and Select Edit Parts

    let mut edit_script_def   = quote!{};
    let edit_script_mets ;
    let edit_script_trts ;

    let mut edit_live_def  = quote!{};
    let edit_live_mets ;
    let edit_live_trts ;


    match aaa.edit {

        crate::attribute::AAEdit  { live, script } => {
            match script {

                ( def , mets, trts) => {
                    if def {
                        edit_script_def = script_def.clone();
                        script_def      = quote!{}; 
                    }
                    edit_script_mets = edit_select(mets,&mut script_mets);
                    edit_script_trts = edit_select(trts,&mut script_trts);
                },
            }

            match live {

                ( def , mets, trts) => {
                    if def {
                        edit_live_def = live_def.clone();
                        live_def      = quote!{}; 
                    }
                    edit_live_mets = edit_select(mets,&mut live_mets);
                    edit_live_trts = edit_select(trts,&mut live_trts);
                },
            }
        }
    }

    // Prepare Token Stream Vecs
    let script_methods = script_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let script_traits  = script_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_methods   = live_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    let live_traits    = live_trts.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
    

    let res_code = quote! {

        #item

        #script_def
        impl #impl_generics #script_name #ty_generics #where_clause {
            #(#script_methods)*
        }
        #(#script_traits)*

        #live_def
        impl #impl_generics #live_name #ty_generics #where_clause {
            #(#live_methods)*
        }
        #(#live_traits)*

    };


    let res_edit_script_mets =  
    if  edit_script_mets.is_empty() { quote!{} }
    else { quote!{ 
        impl #impl_generics #script_name #ty_generics #where_clause {
            #(#edit_script_mets)* 
        }
    }};

    let res_edit_script_trts =  
    if  edit_script_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_script_trts)* }};

    let res_edit_live_mets =  
    if  edit_live_mets.is_empty() { quote!{} }
    else { quote!{ 
        impl #impl_generics #live_name #ty_generics #where_clause { 
            #(#edit_live_mets)* 
        }
    }};

    let res_edit_live_trts =  
    if  edit_live_trts.is_empty() { quote!{} }
    else { quote!{ #(#edit_live_trts)* }};


    let res_edit = quote!{

        #edit_script_def
        #res_edit_script_mets
        #res_edit_script_trts

        #edit_live_def
        #res_edit_live_mets
        #res_edit_live_trts
    };

    (res_code, res_edit)

}



pub fn edit_select(edit_idents: Option<Vec<Ident>>, 
                    ident_mets: &mut Vec<(Ident,TokenStream)> ) -> Vec<TokenStream> {

    let mut res = Vec::new();

    if let Some(idents) = edit_idents { 

        if idents.is_empty() {
            res = ident_mets.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
            ident_mets.clear();
        }

        for ident in idents {
            if let Some(pos) = ident_mets.iter().position(|x| x.0 == ident){
                let (_,trt)  = ident_mets.remove(pos);
                res.push(trt);
            } else {
                let msg = format!("No method named `{}` in Actor's methods.",ident.to_string());
                abort!(ident,msg);
            }
        }
    } 
    res
}


pub fn channels( lib: &AALib,
             channel: &AAChannel,
           cust_name: &Ident,
            generics: &Option<syn::TypeGenerics<'_>>) -> ( 
                                    TokenStream,
                                    TokenStream,
                                    TokenStream,
                                    TokenStream,
                                    Box<dyn Fn(Box<Type>) -> TokenStream>,
                                    TokenStream,
                                    TokenStream ){

    let live_field_sender:   TokenStream;
    let play_input_receiver: TokenStream;
    let new_live_send_recv:  TokenStream;
                            
    let type_ident = &name::script(cust_name);
    let (error_live_send,error_live_recv) = error::live_send_recv(cust_name);
    
    let mut live_meth_send_recv = 
        quote!{ let ( send, recv ) = oneshot::channel(); };

    let mut script_field_output: Box<dyn Fn(Box<Type>) -> TokenStream> =
        Box::new(|out_type: Box<Type>|quote!{ output: oneshot::Sender<#out_type>, }); 
    
    let mut live_send_input: TokenStream =
        quote!{let _ = self.sender.send(msg).await;};


    let mut live_recv_output: TokenStream = 
        quote!{ recv.await.expect(#error_live_recv)};

    match  channel {

        AAChannel::Unbounded    => {

            match  lib { 

                AALib::Std      => {
                    live_field_sender   = quote!{ sender: std::sync::mpsc::Sender<#type_ident #generics>, };   
                    play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#type_ident #generics>, }; 
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::channel(); };
                    live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
                    live_recv_output    = quote!{ recv.recv().expect(#error_live_recv)};
                },

                AALib::Tokio    => {
                    live_field_sender   = quote!{ sender: tokio::sync::mpsc::UnboundedSender<#type_ident #generics>, };
                    play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::UnboundedReceiver<#type_ident #generics>, }; 
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::unbounded_channel(); }; 
                    live_meth_send_recv = quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
                    script_field_output = Box::new(|out_type: Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
                    live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
                },

                AALib::AsyncStd  => {
                    live_field_sender   = quote!{ sender: async_std::channel::Sender<#type_ident #generics>, };
                    play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#type_ident #generics>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::unbounded(); };                    
                },

                AALib::Smol      => {
                    live_field_sender   = quote!{ sender: async_channel::Sender<#type_ident #generics>, };
                    play_input_receiver = quote!{ receiver: async_channel::Receiver<#type_ident #generics>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) =  async_channel::unbounded(); }; 
                },
            }
        },
        AAChannel::Buffer(val)    => {

            match  lib { 

                AALib::Std      => {
                    live_field_sender   = quote!{ sender: std::sync::mpsc::SyncSender<#type_ident #generics>, };
                    play_input_receiver = quote!{ receiver: std::sync::mpsc::Receiver<#type_ident #generics>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = std::sync::mpsc::sync_channel(#val); };
                    live_send_input     = quote!{ let _ = self.sender.send(msg).expect(#error_live_send);};
                    live_recv_output    = quote!{ recv.recv().expect(#error_live_recv)};
                },
                AALib::Tokio    => {
                    live_field_sender   = quote!{ sender: tokio::sync::mpsc::Sender<#type_ident #generics>, };
                    play_input_receiver = quote!{ mut receiver: tokio::sync::mpsc::Receiver<#type_ident #generics>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = tokio::sync::mpsc::channel(#val); }; 
                    live_meth_send_recv = quote!{ let ( send, recv ) = tokio::sync::oneshot::channel(); };
                    script_field_output = Box::new(|out_type: Box<Type>|quote!{ output: tokio::sync::oneshot::Sender<#out_type>, });                
                },

                AALib::AsyncStd  => {
                    live_field_sender   = quote!{ sender: async_std::channel::Sender<#type_ident #generics>, };
                    play_input_receiver = quote!{ receiver: async_std::channel::Receiver<#type_ident #generics>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = async_std::channel::bounded(#val); };
                },

                AALib::Smol      => {
                    live_field_sender   = quote!{ sender: async_channel::Sender<#type_ident #generics>, };
                    play_input_receiver = quote!{ receiver: async_channel::Receiver<#type_ident #generics>, };
                    new_live_send_recv  = quote!{ let ( sender, receiver ) = async_channel::bounded(#val); };
                },
            }
        },
        AAChannel::Inter  => {

            live_field_sender   = quote!{ 
                queue: std::sync::Arc<std::sync::Mutex<Option<Vec<#type_ident #generics>>>>,
                condvar:                       std::sync::Arc<std::sync::Condvar>,
            };
            play_input_receiver = quote!{ 
                queue: std::sync::Arc<std::sync::Mutex<Option<Vec<#type_ident #generics>>>>,
                condvar:                       std::sync::Arc<std::sync::Condvar>,
            };
            new_live_send_recv  = quote!{ 
                let queue       = std::sync::Arc::new(std::sync::Mutex::new(Some(vec![])));
                let condvar     = std::sync::Arc::new(std::sync::Condvar::new());
            };

            let error_msg = error::live_guard(cust_name);
            live_send_input     =  quote!{
                {
                    let mut guard = self.queue.lock().expect(#error_msg);
        
                    guard.as_mut()
                    .map(|s| s.push(msg));
                }
                self.condvar.notify_one();
            };

            live_recv_output     =  quote!{recv.recv().expect(#error_live_recv)};
        },
    }

    
    (
        live_field_sender,
        play_input_receiver, 
        new_live_send_recv , 
        live_meth_send_recv, 
        script_field_output, 
        live_send_input,
        live_recv_output,
    )
}





