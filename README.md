
# All In Parachain
All In Network es una parachain que provee toda la infraestructura y herramientas necesarias para conectar nuevos traders con traders expertos de forma transparente y descentralizada.

## cómo ejecutar el proyecto
### Configuracion de Rust

primero, complete [instrucciones de configuracion de rust](./docs/rust-setup.md).

### Ejecutar

Use el comando nativo de Rust `cargo` para ejecutar el nodo:

```sh
cargo run --release -- --dev
```

### Compilar

Use el siguiente comando para compilar el nodo sin necesidad de ejecutarlo

```sh
cargo build --release
```

### Ejecutar nodo en desarrollo
ejecutar el nodo sin persistir el estado

```bash
./target/release/all-in-network --dev
```

eliminar el estado de la cadena:

```bash
./target/release/all-in-network purge-chain --dev
```

ejecutar el nodo con detalles de logging:

```bash
RUST_BACKTRACE=1 ./target/release/all-in-network -ldebug --dev
```


### Conectar con el Front-end de Polkadot-JS  

Una vez el nodo este ejecutandose localmente, usted puede conectarse a al fron-end de **Polkadot-JS** para interactuar con la cadena
[Click
aqui](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) para conectar el nodo
