# Meld Architecture

Meld spesification is should by compiler and mostly used as a pre-processing features.

TEF specification is used by compiler to emit a correct bytecode and used by runtime to correctly implement the flow of the runtime.

```mermaid
graph TD;
    MP[Meld Project] --> P[Pages];
    P -- Process it at parallel and per-page --> A[Meld Page];
    A -- Pre-processing --> B{Meld & Island Component};
    B --> M(Meld);
    B --> MX(MDx);
    B --> V(Vue);
    B --> R(React);
    B --> S(Svelte);

    C[Compiler];

    M --> C;
    MX --> C;
    V --> C;
    R --> C;
    S --> C;

    C -- emit a single template --> D[TEF Source Template]
    D --> E[Compiler];
    E -- emit bytecode--> F[TEF Bytecode];
```

When we talking about Meld specification we talking about from Meld Project into TEF Source Template.
And when talking about TEF we talking about transpiled TEF Source Template into TEF Bytecode.

Below is how we execute from TEF Bytecode into a target file and it's use TEF spec.

```mermaid
graph TD;
    A[TEF Bytecode] --> R[Runtime];
    R -- execute --> B[Target File]
```
