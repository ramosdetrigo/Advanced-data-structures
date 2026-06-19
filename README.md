# Estrutura de van Emde Boas
Aluna: Júlia Andrade Ramos \
Matrícula: 558279

# Linguagem de programação usada:
Todo o código foi feito na última versão estável de Rust (1.85.0 – Rust 2024). \
É recomendado seguir o [manual de instalação oficial da linguagem](https://rust-lang.org/tools/install/)

# Como rodar:
Uma vez instalada a toolchain oficial de Rust é só rodar `cargo run <input>` para buildar _e_ rodar o projeto. (ou `cargo build` só para buildar.)

Também foi feito um makefile que faz a mesma coisa: `make build` & `make run <input>`

O input é o caminho para o arquivo de comandos, relativo à pasta onde o projeto foi executado (ou um caminho global).

# Descrição do projeto
A estrutura geral da árvore vEB e as funções principais (`include`, `remove`, `successor`, `predecessor`, `imp`) estão no arquivo referente ao módulo `veb/mod.rs`. Uma estrutura auxiliar simples de Hash Map com doubling e halving foi implementada do zero no arquivo `veb/hashmap.rs` para manter o número de elementos em espaço linear. Como a árvore de Van Emde Boas depende estritamente da manipulação de bits, toda a estrutura foi implementada para trabalhar com o tipo de inteiros sem sinal de 32 bits (`u32`).

Estrutura do comando para rodar: \
`make run INPUT=<file path>` ou `cargo run <file path>` \
Saída: As impressões são mostradas diretamente no terminal. \
O arquivo de entrada aceita comentários iniciados com `#`, como em python. \
Toda a implementação da interface entre a estrutura e a leitura do arquivo está na `src/main.rs`, tratando os comandos `INC`, `REM`, `SUC`, `PRE` e `IMP`. Casos onde o sucessor ou predecessor não existem são formatados automaticamente para imprimir `+INF` ou `-INF`, conforme especificado.

# Estruturas e funções
## Tipos e constantes:
### `INIT_CAP = 16`
> Constante - Capacidade inicial dos buckets no hashmap customizado.

---
## Estruturas principais:

### Veb:
- **w**: `u32`
  > O tamanho do universo daquele nó da árvore em bits (começa com 32 e vai caindo pela metade; 16, 8, 4, 2, 1).
- **min**: `Option<u32>`
  > O valor mínimo armazenado na árvore.
- **max**: `Option<u32>`
  > O valor máximo armazenado na árvore (deve ser tratado como uma cópia, diferente do `min`). Este valor pode estar nos clusters, mas aqui serve para checagem em O(1)). A única exceção é quando `w = 1`, onde a árvore é tratada como "folha" e só armazena o máximo e mínimo diretamente.
- **clusters**: `VEBHashMap`
  > Hash Map que mapeia os bits mais altos para a sub-árvore correspondente.
- **resumo**: `OnceCell<Box<Veb>>`
  > Uma estrutura para a árvore de resumo. O resumo guarda quais clusters estão ocupados. O `OnceCell` garante que a sub-árvore de resumo só será alocada na memória (na _heap_ via `Box`) quando necessário, economizando espaço. Acessado via método privado de mesmo nome: `resumo()` e `resumo_mut()`

#### Métodos principais:
- `include(x: u32)`
  > Insere um elemento na estrutura vEB. Se a árvore está vazia, define `min` e `max`. Caso contrário, checa se não é duplicata. Se `x` for menor que o `min` atual, eles são trocados, e a inserção continua com o antigo `min`. O valor é quebrado em dois `<c,i>` através da função `split_bits`. O resumo e os clusters são inicializados sob demanda e o valor é inserido recursivamente.
- `remove(x: u32)`
  > Remove um elemento. Se o elemento for o `min`, a função busca o próximo menor elemento através do `resumo` para promovê-lo a novo `min`, e então remove o valor correspondente do cluster em que ele estava. Se um cluster ficar vazio, ele é deletado do resumo e da memória. Também cuida de atualizar o `max` de acordo.
- `successor(x: u32) -> Option<u32>`
  > Retorna o próximo valor estritamente maior que `x`. Checa o `min` e o `max` em O(1). Se a resposta puder estar no mesmo cluster de `x`, busca lá dentro; senão, busca o próximo cluster não-vazio no `resumo` e retorna o mínimo desse próximo cluster. Retorna `None` se não houver sucessor.
- `predecessor(x: u32) -> Option<u32>`
  > O simétrico do sucessor. Retorna o valor estritamente menor que `x`. Checa se a resposta está dentro do próprio cluster de `x`; caso contrário, consulta o `resumo` para achar o cluster ocupado anterior e pega o `max` de lá. Se não existirem clusters anteriores ocupados, o predecessor é o `min` da árvore atual.
- `imp() -> String`
  > Retorna uma `String` formatada representando os elementos da árvore de forma hierárquica. Agrupa os valores em seus respectivos clusters formatados (ex: `C[1]: 2, 3`), fazendo chamadas recursivas via método auxiliar `collect()`.

---

### VEBHashMap:
Uma implementação própria de tabela de dispersão (Hash Map) utilizando tratamento de colisão por encadeamento. Feita especificamente para chaves `u32`.
- **buckets**: `Vec<Vec<(u32, Veb)>>`
  > Vetor de vetores armazenando tuplas de `(chave, árvore_vEB)`.
- **len**: `usize`
  > Quantidade de chaves armazenadas no hashmap.

#### Métodos principais:
- `insert(key: u32, value: Veb)`
  > Adiciona uma nova sub-árvore associada a uma chave (índice do cluster). Também gerencia o crescimento dinâmico da tabela: se o uso passar de 3/4 da capacidade total, chama a função privada `double()` que aloca o dobro de espaço e faz o re-hashing das chaves em O(N).
- `remove(key: u32) -> Option<Veb>`
  > Remove e retorna a sub-árvore referente àquela chave utilizando `swap_remove` para deleção em O(1) no vetor de colisão. Caso o tamanho da tabela caia para 1/4 da capacidade, chama a função `halve()` para encolher a tabela pela metade.
- `get(key: u32)` e `get_mut(key: u32)`
  > Buscam e retornam referências (imutáveis ou mutáveis) para um cluster através da sua chave.

---
## Estruturas auxiliares e Funções de Bits:
As funções de manipulação de bits substituem contas matemáticas caras por deslocamentos lógicos super eficientes.
- `split_bits(x: u32, w: u32) -> (u32, u32)`
  > Quebra o número `x` no meio (relativo a `w`), retornando os bits mais altos e os mais baixos `<c, i>`.
- `lower_bits` e `higher_bits`
  > Executam as operações bit a bit em si utilizadas no split.
- `merge_bits(c: u32, i: u32, w: u32) -> u32`
  > Remonta um valor original a partir de seu número de cluster `c` e do índice interno `i`.
