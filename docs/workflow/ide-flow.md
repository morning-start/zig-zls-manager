# IDE 集成流程 - Zig/ZLS 版本管理器

## IDE 集成流程总览

```mermaid
flowchart TD
    A[用户执行: zzm ide <subcommand>] --> B[解析子命令]
    B -->|config| C[配置 IDE]
    B -->|check| D[检查配置]
    B -->|doctor| E[诊断工具]
    B -->|path| F[显示路径]
    
    C --> G{选择 IDE}
    G -->|vscode| H[生成 VS Code 配置]
    G -->|neovim| I[生成 Neovim 配置]
    G -->|helix| J[生成 Helix 配置]
    G -->|jetbrains| K[生成 JetBrains 配置]
    
    H --> L[获取 Zig/ZLS 路径]
    I --> L
    J --> L
    K --> L
    L --> M[生成配置文件]
    M --> N[写入目标位置]
    N --> O[输出成功信息]
    
    D --> P[检测当前 IDE]
    P --> Q[读取现有配置]
    Q --> R[验证配置有效性]
    R --> S[输出检查结果]
    
    E --> T[运行全面诊断]
    T --> U[检查安装状态]
    U --> V[检查 PATH 配置]
    V --> W[检查 IDE 配置]
    W --> X[输出诊断报告]
    
    F --> Y[获取工具路径]
    Y --> Z[格式化输出]
```

## IDE 配置生成子流程

### 2.1 VS Code 配置生成

```mermaid
flowchart TD
    A[开始配置 VS Code] --> B[获取当前活动版本]
    B --> C[获取 Zig 路径]
    B --> D[获取 ZLS 路径]
    C --> E[生成 settings.json]
    D --> E
    E --> F{项目级配置?}
    F -->|是| G[.vscode/settings.json]
    F -->|否| H[用户设置目录]
    G --> I[写入配置文件]
    H --> I
    I --> J[提示重启 VS Code]
```

### 2.2 Neovim 配置生成

```mermaid
flowchart TD
    A[开始配置 Neovim] --> B[获取 Zig/ZLS 路径]
    B --> C{检查 lspconfig}
    C -->|已安装| D[生成 lspconfig 配置]
    C -->|未安装| E[提示安装 lspconfig]
    D --> F[生成 init.lua 片段]
    F --> G[输出配置内容]
    G --> H[提示用户复制到配置文件]
```

### 2.3 Helix 配置生成

```mermaid
flowchart TD
    A[开始配置 Helix] --> B[检查 Helix 配置目录]
    B --> C{languages.toml 存在?}
    C -->|是| D[读取现有配置]
    C -->|否| E[创建新文件]
    D --> F[添加/更新 Zig 配置]
    E --> F
    F --> G[写入 languages.toml]
    G --> H[提示 Helix 会自动检测 PATH]
```

## IDE 诊断流程

```mermaid
flowchart TD
    A[开始诊断] --> B[检查 Zig 安装]
    B --> C{Zig 已安装?}
    C -->|否| D[标记问题: Zig 未安装]
    C -->|是| E[检查 Zig 版本]
    E --> F[检查 ZLS 安装]
    D --> F
    
    F --> G{ZLS 已安装?}
    G -->|否| H[标记问题: ZLS 未安装]
    G -->|是| I[检查 ZLS 版本]
    H --> J
    
    I --> J[兼容性检查]
    J --> K{兼容?}
    K -->|否| L[标记问题: 版本不兼容]
    K -->|是| M[检查 PATH]
    L --> M
    
    M --> N{路径正确?}
    N -->|否| O[标记问题: PATH 配置错误]
    N -->|是| P[检查 IDE 配置]
    O --> P
    
    P --> Q{配置正确?}
    Q -->|否| R[标记问题: IDE 配置错误]
    Q -->|是| S[全部通过]
    R --> T[生成报告]
    S --> T
```

## IDE 集成状态机

```mermaid
stateDiagram-v2
    [*] --> Parsing
    Parsing --> Config : config 子命令
    Parsing --> Check : check 子命令
    Parsing --> Doctor : doctor 子命令
    Parsing --> Path : path 子命令
    
    Config --> IdeSelect : 选择 IDE
    IdeSelect --> VSC : VS Code
    IdeSelect --> Neo : Neovim
    IdeSelect --> Helix : Helix
    IdeSelect --> JB : JetBrains
    
    VSC --> Generating : 生成配置
    Neo --> Generating
    Helix --> Generating
    JB --> Generating
    
    Generating --> Writing : 配置生成完成
    Writing --> Done : 写入完成
    
    Check --> Detecting : 检测 IDE
    Detecting --> ReadingConfig : 读取配置
    ReadingConfig --> Validating : 验证配置
    Validating --> Done : 输出结果
    
    Doctor --> FullCheck : 全面检查
    FullCheck --> Done : 生成报告
    
    Path --> GettingPaths : 获取路径
    GettingPaths --> Done : 输出路径
    
    Done --> [*]
```

## 配置文件格式示例

### VS Code settings.json

```json
{
  "zig.path": "~/.zzm/bin/zig",
  "zig.zls.path": "~/.zzm/bin/zls",
  "[zig]": {
    "editor.defaultFormatter": "ziglang.vscode-zig",
    "editor.formatOnSave": true
  }
}
```

### Neovim lspconfig

```lua
require('lspconfig').zls.setup({
  cmd = { '~/.zzm/bin/zls' },
  settings = {
    zls = {
      zig_exe_path = '~/.zzm/bin/zig',
      enable_inlay_hints = true,
    }
  }
})
```

### Helix languages.toml

```toml
[language-server.zls]
command = "zls"

[[language]]
name = "zig"
language-servers = ["zls"]
formatter = { command = "zig", args = ["fmt", "--stdin"] }
```