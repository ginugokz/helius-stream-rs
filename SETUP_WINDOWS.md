# Setup Windows — resolver `dlltool.exe not found`

## O que está a acontecer

O erro:

```
error: error calling dlltool 'dlltool.exe': program not found
error: could not compile `getrandom` (lib) due to 1 previous error
```

Significa: a tua toolchain Rust está configurada para o target **Windows-GNU** (sufixo `-gnu`), que usa o linker GCC/binutils. As crates `getrandom v0.3` e `windows-sys v0.61` invocam `dlltool` (parte do MinGW binutils) para gerar import libraries do Windows. O `dlltool.exe` não está no teu PATH.

Não é problema da crate. É problema da toolchain Windows do Rust. Há **dois caminhos** para resolver.

## Verificar primeiro

```powershell
rustup show
```

Se vires algo como `stable-x86_64-pc-windows-gnu` como default, confirma que estás em GNU.

---

## Caminho A (recomendado) — Switch para toolchain MSVC

Funciona sem MSYS2, é o setup-padrão da maioria dos devs Rust em Windows.

### Pré-requisito

**Visual Studio Build Tools 2022** com o workload "Desktop development with C++".
Já tens (vê em "Apps & features" no Windows ou corre `where cl.exe`)?
- Se sim: salta para "Switch toolchain".
- Se não: instala daqui — https://visualstudio.microsoft.com/visual-cpp-build-tools/
  - Download é ~6 GB; instala apenas o workload "Desktop development with C++" (não precisas do IDE inteiro).

### Switch toolchain

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup show
```

Confirma que o output diz `x86_64-pc-windows-msvc` em "default toolchain".

### Limpa e rebuild

```powershell
cd C:\Users\ginug\Documents\Claude\Projects\NEW\helius_stream_rs
cargo clean
cargo build
```

Deve compilar limpo.

---

## Caminho B — Manter GNU, instalar MinGW binutils

Faz sentido se já usas MSYS2 noutros projetos ou se queres evitar MSVC.

### Instalar MSYS2

Download: https://www.msys2.org/
Run o installer com defaults. Depois abre um terminal MSYS2 MINGW64 e:

```bash
pacman -Syu                                        # update repos
pacman -S mingw-w64-x86_64-binutils \
          mingw-w64-x86_64-gcc \
          mingw-w64-x86_64-pkg-config              # instala dlltool + gcc + pkg-config
```

### Adicionar ao PATH

Em PowerShell, permanentemente (precisa de Admin):

```powershell
[Environment]::SetEnvironmentVariable("Path",
    [Environment]::GetEnvironmentVariable("Path", "User") + ";C:\msys64\mingw64\bin",
    "User")
```

Fecha e reabre o terminal. Confirma:

```powershell
where dlltool        # devolve C:\msys64\mingw64\bin\dlltool.exe
where gcc            # devolve C:\msys64\mingw64\bin\gcc.exe
```

### Limpa e rebuild

```powershell
cd C:\Users\ginug\Documents\Claude\Projects\NEW\helius_stream_rs
cargo clean
cargo build
```

---

## Sanity check do exemplo

PowerShell **não aceita** sintaxe `VAR=value command` (isso é Unix/bash/cmd.exe). Em PowerShell:

```powershell
$env:HELIUS_API_KEY = "a-tua-key-helius"
cargo run --example basic_stream
```

Ou em uma linha:

```powershell
$env:HELIUS_API_KEY="a-tua-key-helius"; cargo run --example basic_stream
```

Deve imprimir 10 updates da USDC mint e sair.

---

## Como prometheus9 compilou e isto não

Provavelmente o prometheus9 foi compilado anteriormente noutro setup (talvez MSVC, ou com MSYS2 instalado na altura) e o binário ficou em cache em `target/release/`. Se correres `cargo clean && cargo build` no prometheus9 agora, vais ver o mesmo erro.

Resolver isto uma vez (Caminho A ou B) destranca todas as crates Rust que toques no futuro.
