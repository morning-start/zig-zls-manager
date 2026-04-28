# 兼容性检查流程 - Zig/ZLS 版本管理器

## 兼容性检查流程总览

```mermaid
flowchart TD
    A[触发兼容性检查] --> B[获取当前 Zig 版本]
    B --> C[获取当前 ZLS 版本]
    C --> D[加载兼容性矩阵]
    D --> E{矩阵需要更新?}
    E -->|是| F[从远程更新矩阵]
    E -->|否| G[使用本地矩阵]
    F --> G
    G --> H[查找匹配规则]
    H --> I{找到规则?}
    I -->|否| J[标记为未知]
    I -->|是| K[检查兼容状态]
    K --> L{状态类型}
    L -->|完全兼容| M[输出成功信息]
    L -->|可能兼容| N[输出警告]
    L -->|不兼容| O[输出错误]
    L -->|未知| J
    J --> P[输出建议]
    M --> P
    N --> P
    O --> P
```

## 兼容性矩阵查询流程

```mermaid
flowchart TD
    A[开始查询] --> B[解析 Zig 版本]
    B --> C[提取主版本号]
    C --> D[遍历兼容性规则]
    D --> E{版本范围匹配?}
    E -->|是| F[记录匹配规则]
    E -->|否| G{还有规则?}
    F --> G
    G -->|是| D
    G -->|否| H{找到匹配?}
    H -->|是| I[返回匹配规则]
    H -->|否| J[返回未知]
```

## 兼容性状态判定

```mermaid
flowchart TD
    A[获取 Zig 和 ZLS 版本] --> B[检查完全匹配]
    B --> C{推荐版本匹配?}
    C -->|是| D[完全兼容]
    C -->|否| E[检查已知兼容列表]
    E --> F{ZLS 在列表中?}
    F -->|是| G[可能兼容]
    F -->|否| H[检查已知问题]
    H --> I{存在已知问题?}
    I -->|是| J[不兼容]
    I -->|否| K[未知]
```

## 兼容性矩阵结构

```mermaid
flowchart LR
    A[兼容性矩阵] --> B[规则列表]
    B --> C[规则 1]
    B --> D[规则 2]
    B --> E[规则 N]
    
    C --> C1[Zig 版本范围]
    C --> C2[推荐 ZLS]
    C --> C3[已知兼容列表]
    C --> C4[已知问题]
    C --> C5[验证时间]
    
    D --> D1[Zig 版本范围]
    D --> D2[推荐 ZLS]
    D --> D3[已知兼容列表]
    D --> D4[已知问题]
    D --> D5[验证时间]
```

## 兼容性检查状态机

```mermaid
stateDiagram-v2
    [*] --> LoadingMatrix
    LoadingMatrix --> CheckingUpdate : 加载完成
    
    CheckingUpdate --> RemoteUpdate : 需要更新
    CheckingUpdate --> Querying : 使用本地
    
    RemoteUpdate --> Querying : 更新完成
    
    Querying --> FoundRule : 找到匹配规则
    Querying --> Unknown : 未找到规则
    
    FoundRule --> FullyCompatible : 完全兼容
    FoundRule --> PossiblyCompatible : 可能兼容
    FoundRule --> Incompatible : 不兼容
    
    FullyCompatible --> Done : 输出成功
    PossiblyCompatible --> Done : 输出警告
    Incompatible --> Done : 输出错误
    Unknown --> Done : 输出建议
    
    Done --> [*]
```

## 兼容性矩阵更新流程

```mermaid
flowchart TD
    A[检查更新] --> B[获取本地版本]
    B --> C[请求远程版本]
    C --> D{远程版本更新?}
    D -->|否| E[使用本地矩阵]
    D -->|是| F[下载新矩阵]
    F --> G[验证矩阵格式]
    G --> H{格式有效?}
    H -->|否| I[保留旧矩阵]
    H -->|是| J[保存新矩阵]
    J --> E
    I --> E
```

## 兼容性规则示例

| Zig 版本 | 推荐 ZLS | 已知兼容 | 已知问题 | 验证时间 |
|---------|---------|---------|---------|---------|
| 0.11.* | 0.11.0 | 0.11.0 | 无 | 2026-04-01 |
| 0.12.* | 0.12.0 | 0.12.0 | 无 | 2026-04-01 |
| 0.13.* | 0.13.0 | 0.13.0 | master 分支可能不稳定 | 2026-04-25 |
| master | nightly | nightly | 可能存在兼容性问题 | 每日更新 |