#import "@preview/hydra:0.6.3": hydra
#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#import "@preview/wordometer:0.1.5": word-count, total-words

#show: codly-init.with()
#show: word-count

#set document(title: [How good is Rust at compiling C?])
#set heading(numbering: 
              (first, ..rest) => 
              if rest.pos().len() == 0 {
                return str(first) + "."
              } else {
                return str(first) + "." + rest.pos().map(str).join(".")
              }
            ) 

#set page(paper: "us-letter", header: context {
  if counter(page).get().first() > 1 [
    #hydra(1)
    #h(1fr)
    #counter(page).display("1")
  ]
})

#align(center + horizon, [
  #title()
  Leon McQueen\
  Hampton School, L6DCW \
  #total-words
])
#pagebreak()

= Introduction

The history of the general-purpose programming language is a long and complicated one.
Over more than seventy-five years of development, thousands of coding languages
have been created, all with different factors motivating their subsequent development.
In the modern era of language design, commonly used languages tend to separate
themselves into two categories.

High level languages like C\# or Java are popular because of their ease of use and
simplicity, as well as their deep integration into various platforms, like C\#'s
integration into Microsoft's .NET ecosystem. However, some programmers and engineers often 
regard these languages as bloated @frustrated-dot-net-c-sharp
due to their abstraction of features away from the user, as 
well as the overuse of memory due to garbage collection, often resulting in a long and 
arduous journey to an optimized program.

Lower-level languages like C or C++ allow the user finer control over 
features which are often abstracted by higher level languages. This means that
further optimizations #footnote([like asynchronous I/O or zero-copy techniques; see
@speed-optimizations]) 
can be made to a program. With this functionality, 
however, comes increased complexity and verbosity, as well as a larger risk of
issues like memory leaks and buffer overflows due to the need for the user to
manually allocate and deallocate memory for variables.

The compiler made for this project was built in Rust, a relatively new programming language,
with an initial stable release being published "only" eleven years ago, in May of
2015 @rust-1-0, and a latest stable release of Rust 1.97.1 as of July 21, 2026 
@rust-latest-release. Rust describes itself as "blazingly fast and memory-efficient", due to 
its intermediate compilation to the LLVM IR and the language's lifetimes feature, 
where the user writes annotations to establish control over the time that a reference
is in scope. This ensures memory safety throughout the runtime of a program by
preventing dangling references, since there is a natural prevention of issues like memory 
leaks via the design of the language itself.

#pagebreak()

= bits and pieces

Having built an operating system in Rust previously, I was interested to explore its
capabilities further. I had recently heard of Rust being integrated into the Linux
kernel, and through several YouTube videos had discovered more and more of the
capabilities of the language, which piqued my interest of the other capabilities of the
language, outside what I had learnt while building the operating system. Additionally, I had
been increasingly interested in compilers and machine code (through a brief attempt
at building a bootloader in assembly for the operating system)
#pagebreak()
#bibliography("sources/bib.yaml", style: "ieee")
