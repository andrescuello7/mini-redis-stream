# Redis STM

Redis STM es una implementacion basica de un servidor tipo Redis escrita en Rust con Tokio.
El proyecto busca servir como ejercicio de aprendizaje para entender como funciona un servidor TCP asincrono, como se aceptan multiples conexiones concurrentes y como se puede modelar una base de datos en memoria.

## Proposito

La idea principal de este repositorio es construir una version pequena y educativa de Redis, enfocada en:

- aprender programacion asincrona con Rust y Tokio;
- entender el manejo de conexiones TCP concurrentes;
- experimentar con una arquitectura simple de comandos y almacenamiento en memoria;
- tener una base sobre la cual agregar comandos como `GET` y `SET`.

## Estado actual

Actualmente el proyecto levanta un servidor TCP en `127.0.0.1:6379` y contiene la estructura inicial para manejar conexiones y comandos.
Todavia esta en una etapa temprana, por lo que varias piezas del comportamiento tipo Redis siguen en construccion.

## Tecnologias

- Rust
- Tokio
- Tracing

## Ejecucion local

Para iniciar el servidor:

```bash
cargo run
```

Si el proceso inicia correctamente, el servidor quedara escuchando en el puerto `6379`.

## Objetivo del proyecto

Este repositorio no busca reemplazar Redis real.
Su objetivo es didactico: entender su modelo general de funcionamiento y construir una implementacion propia, pequena y extensible.
