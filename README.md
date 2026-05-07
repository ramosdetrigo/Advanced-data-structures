# Árvore binária com persistência parcial
Aluno: Júlia Andrade Ramos \
Matrícula: 558279

# Linguagem de programação usada:
Todo o código foi feito na última versão estável de Rust (1.85.0 - Rust 2024). \
É recomendado seguir o [manual de instalação oficial da linguagem](https://rust-lang.org/tools/install/)

# Como rodar:
Uma vez instalada a toolchain oficial de Rust (o principal é só a ferramenta `cargo`), é só rodar `cargo run <input>` pra buildar _e_ rodar o projeto. (ou `cargo build` só pra buildar.)

Também foi feito um makefile que faz a mesma coisa: `make build` & `make run <input>`

O input é só o caminho pro arquivo relativo à pasta em que você rodou o projeto. (Pode ser o caminho global também).

# Descrição do projeto
A estrutura geral da árvore e as funções de push, remove, successor e print estão no arquivo `src/persistent_btree/mod.rs`. A estrutura e as funções relacionadas ao nó, modificações, etc. estão no arquivo `src/persistent_btree/node.rs`. A estrutura inteira foi escrita de forma que ela pode ordenar qualquer tipo genérico ordenável `T`. Esse é o `<T>` escrito nas especificações abaixo. Para todo propósito prático você pode considerar ele como um tipo `int`. \
Adicionalmente, o arquivo de entrada aceita comentários! Basta fazer um comentário como em python: `# Assim! Começando com "#"`. Pode ajudar com debugging :) \
Toda a implementação da interface entre a estrutura e a leitura do arquivo está na `src/main.rs`, e é até que bem simples de entender pelos comentários.

# Estruturas e funções
## Tipos e constantes:
### `NodePtr<T> = NonNull<Node<T>>`
> Uma abstração de um ponteiro garantidamente não-nulo para um nó.

### `Link<T> = Option<NodePtr<T>>`
> Uma abstração para um link para outro nó: Um `Option` é uma ferramenta da linguagem que facilita (e explicita) lidar com os casos onde dados podem ser nulos.

### `P = 3`
> Constante - número de ponteiros de um nó.

---
## Estruturas principais:

### PBTree:
- **root_history**: `Vec<(Link<T>, usize)>`
  > Vetor de raízes da árvore ao longo das versões dela. Armazena tuplas (nó, versão).
- **version**: `usize`
  > Inteiro da última versão da árvore.
- **_t**: `PhantomData<T>`
  > Marcador de tamanho 0 interno da linguagem para ajudar o compilador na questão de gerenciamento de memória etc.

#### Métodos principais:
- `push(value: T)`
  > Insere um elemento na árvore. Usa a função `find_parent_node` para achar um nó pai candidato para o elemento, fazendo uma busca pela árvore. A aperação de adição em si é feita através do método `add_mod()` do nó encontrado, testando se o nó deve ser um filho esquerdo ou direito e criando uma modificação pra ele. O nó em si já é criado apontando pro pai, sem nenhuma mod adicionado na sua lista de mods. No caso onde não é encontrado um pai, a árvore está vazia e o nó é adicionado como raiz da árvore.
- `remove(value: &T) -> Option<T>`
  > Remove um elemento da árvore. Retorna o elemento se ele existe, retorna `None` e não mexe na árvore se o elemento não existe nela. Faz isso através da função auxiliar `transplant(node, target)`, que transplanta um nó com outro: Tira um nó de baixo do pai e coloca outro no lugar. Todas as mudanças de ponteiros e nós etc. são feitas através do método `add_mod` dos nós. A remoção em si tem apenas algumas observações: 1. Estamos sempre, após cada modificação do nó sucessor, checando se a modificação resultou na criação de um novo nó. Se sim, o método `add_mod` retorna o ponteiro pro novo nó onde o restante das modificações deverão ser feitas. 2. São removidos os ponteiros de retorno do nó removido (e do nó sucessor) antes do transplant acontecer, para evitar redireções desnecessárias no caso de criação de novo nó em uma adição de modificação. Essas redireções podem inclusive acabar deixando os ponteiros que estamos manejando durante a execução da função desatualizados por mudanças recursivas, etc. Alguns comentários no código explicam melhor.
- `successor(elem: &T, version: usize) -> Option<T>`
  > Retorna o sucessor estritamente maior de um elemento especificado em uma versão específica da estrutura. `None` se o sucessor não existe. \
  > **OBS:** para a interface de leitura de arquivo de entrada foi feita uma tradução para que, se não exista sucessor, o sucessor seja printado como "INFINITO", conforme a especificação.
- `list(version: usize) -> String`
  > Retorna uma string listando os elementos da árvore do menor pro maior, com suas respectivas profundidades, em uma versão específica dela, conforme a especificação.

---

### Node:
- **value**: `T`
  > O valor do nó (pode ser de um tipo ordenável genérico)
- **parent**: `Link<T>`
  > Ponteiro pro pai do nó
- **left**: `Link<T>`
  > Ponteiro pro filho esquerdo do nó
- **right**: `Link<T>`
  > Ponteiro pro filho direito do nó
- **tree_ptr**: `NonNull<PBTree<T>>`
  > Ponteiro pra estrutura da árvore. Utilizado apenas para o caso onde o nó raiz atinge o limite de modificações e cria um novo nó, sendo necessário atualizar a estrutura base da árvore em si.
- **return_pts**: `Vec<ReturnPtr<T>>`
  > Vetor de ponteiros de retorno do nó. É armazenado como um vetor apenas por conveniência. Dentro do código a limitação de P = 3 é imposta via código com uma constante `P = 3`.
- **mods**: `Vec<Modification<T>>`
  > Vetor de modificações do nó. Assim como o vetor de ponteiros, é utilizado apenas por conveniência. A limitação também é imposta via código com `P * 2`

#### Métodos principais:

- `add_mod(modifier: Modification<T>)` e `add_mod_full(modifier: Modification<T>)`
  > Métodos para adicionar modificações nos nós. É sempre utilizado o `add_mod`, o `add_mod_full` é uma função privada auxiliar pro caso onde o campo de modificações está cheio. \
  > Caso base (há espaço para mods): Se for um update de valor, só adiciona ele na lista. Se não, é criado um clone do nó, é feito um match com o enum de modificação e é adicionada a modificação no nó. Após isso, é removido o ponteiro de retorno do nó que estava sendo apontado antes da modificação, e é adicionado um ponteiro de retorno no novo nó apontado. \
  > Caso 2 (não há espaço): É criado um clone do nó alocado na heap. Os ponteiros de retorno que apontavam pro nó antigo agora são redirecionados pro nó novo manualmente (isso previne o mesmo problema que poderia acontecer no `PBTree.remove`), e o ponteiro de retorno redireciona o nó pra onde ele aponta pra apontar pro nó novo através do método `redirect_node` do enum `ReturnPtr`, adicionando os ponteiros de retorno no campo do nó novo no processo. Por fim, a função `add_mod` é chamada no nó recém-criado, e logo após é aplicada diretamente e removida a modificação nele, pros nós que apontam pra ele já apontarem pra nova versão, sem precisar aplicar uma modificação a mais e consumir um espaço de mod desnecessariamente. Por fim é checado apenas se o pai do nó é `None`. Se sim, é adicionado o novo nó como nova raiz na última versão da árvore.
- `modded_clone() -> Node<T>`
  > Retorna uma cópia do nó com todas as modificações no vetor de mods aplicadas e campos de mods/ponteiros de retorno vazios.
- `snapshot(version: usize) -> Node<T>`
  > Retorna uma cópia do nó com todas as modificações até uma versão específica no vetor de mods aplicadas e campos de mods/ponteiros de retorno vazios.
- `apply_mod(modifier: &Modification<T>)`
  > **Aplica** uma modificação diretamente no nó.
- `into_link() -> Link<T>`
  > Armazena o nó na heap e retorna um link pra ele
- `as_ptr() -> NodePtr<T>`
  > Retorna um ponteiro pro nó

---
## Estruturas auxiliares:

### Modification:
- **version**: `usize`
  > A versão em que a modificação foi aplicada
- **modification**: `ModTarget<T>`
  > Os dados da modificação

### ModTarget:
Enum que armazena o alvo e o dado da modificação:
```rust
enum ModTarget<T> {
    Parent(Link<T>),
    Left(Link<T>),
    Right(Link<T>),
    Value(T),
}
```

### ReturnPtr:
Estrutura que armazena e explicita de onde vem o ponteiro de retorno: O nó atual é filho esquerdo de, filho direito de, ou pai de outro nó
```rust
enum ReturnPtr<T> {
    ParentOf(NodePtr<T>),
    LeftOf(NodePtr<T>),
    RightOf(NodePtr<T>),
}
```
**OBS:** esse enum tem um método implementado que permite redirecionar pra onde o nó apontado está apontando (através de um `add_mod`). Só é usado no método `add_mod_full` do Node pra redirecionar os nós que apontavam pro nó antigo pro novo nó.
