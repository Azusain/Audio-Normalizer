# 使用指南

## 命令行参数详解

### 基本语法
```bash
audio_normalizer [选项] 输入文件 [输出文件]
```

### 选项说明

| 选项 | 长选项 | 参数 | 默认值 | 说明 |
|------|--------|------|--------|------|
| `-m` | `--max-peak` | `<dB值>` | `-12.0` | 设置目标峰值电平 |
| `-v` | `--verbose` | 无 | 关闭 | 启用详细输出（调试模式） |
| `-q` | `--quiet` | 无 | 关闭 | 安静模式（仅显示错误） |
| | `--peak` | 无 | 关闭 | 仅分析峰值，不进行标准化 |
| `-h` | `--help` | 无 | | 显示帮助信息 |

## 使用场景

### 1. 基础标准化处理

将音频文件标准化到默认的 -12 dB 峰值：

```bash
audio_normalizer input.wav output.wav
```

**适用场景**: 
- 一般音频标准化需求
- 适合大多数流媒体平台的推荐电平

### 2. 自定义峰值电平

标准化到不同的峰值电平：

```bash
# 标准化到 -6 dB (较高音量)
audio_normalizer -m -6 input.wav output.wav

# 标准化到 -18 dB (较低音量，适合母带处理)
audio_normalizer -m -18 input.wav output.wav

# 标准化到 -3 dB (接近最大音量)
audio_normalizer -m -3 input.wav output.wav
```

**适用场景**:
- `-3 dB`: CD 母带制作、最大音量需求
- `-6 dB`: 音乐制作、混音阶段
- `-12 dB`: 流媒体平台（Spotify、YouTube 等）
- `-18 dB`: 广播电视、专业后期制作
- `-23 dB`: EBU R128 推荐的广播标准

### 3. 分析模式

仅分析文件的峰值电平，不进行处理：

```bash
audio_normalizer --peak input.wav
```

输出示例：
```
Peak level: -8.45 dB
```

**适用场景**:
- 批量文件分析
- 确定是否需要标准化处理
- 质量控制检查

### 4. 详细模式

显示完整的处理信息：

```bash
audio_normalizer -v -m -12 input.wav output.wav
```

输出示例：
```
[2025-08-17 22:45:12.345] [info] Audio Normalizer v1.0.0
[2025-08-17 22:45:12.346] [info] Starting audio normalization...
[2025-08-17 22:45:12.347] [info] Input: input.wav -> Output: output.wav
[2025-08-17 22:45:12.348] [info] Target peak level: -12.00 dB
[2025-08-17 22:45:12.349] [info] Input file info:
[2025-08-17 22:45:12.350] [info]   Sample rate: 44100 Hz
[2025-08-17 22:45:12.351] [info]   Channels: 2
[2025-08-17 22:45:12.352] [info]   Frames: 220500
[2025-08-17 22:45:12.353] [info]   Duration: 5.00 seconds
[2025-08-17 22:45:12.405] [info] Current peak level: -8.45 dB
[2025-08-17 22:45:12.406] [info] Target peak level: -12.00 dB
[2025-08-17 22:45:12.407] [info] Required gain: -3.55 dB (0.661x)
[2025-08-17 22:45:12.452] [info] Output peak level: -12.01 dB
[2025-08-17 22:45:12.453] [info] Normalization completed successfully!
```

**适用场景**:
- 调试和问题诊断
- 学习音频处理过程
- 验证处理结果

### 5. 安静模式

仅在出错时显示信息：

```bash
audio_normalizer -q -m -12 input.wav output.wav
```

**适用场景**:
- 批处理脚本
- 自动化工作流
- 服务器端处理

## 文件格式支持

### 输入格式
- **WAV**: 标准 PCM 和浮点格式
- **FLAC**: 无损压缩格式
- **OGG**: Ogg Vorbis 有损格式
- **AU**: Sun/NeXT 音频格式
- **AIFF**: Apple 音频交换格式

### 输出格式
输出格式由文件扩展名自动确定：

```bash
# 输出为 WAV 格式
audio_normalizer input.flac output.wav

# 输出为 FLAC 格式
audio_normalizer input.wav output.flac

# 输出为 OGG 格式
audio_normalizer input.wav output.ogg
```

## 批处理示例

### Windows 批处理脚本
```batch
@echo off
for %%f in (*.wav) do (
    echo Processing %%f...
    audio_normalizer -q -m -12 "%%f" "normalized_%%f"
)
echo All files processed.
```

### Linux/macOS Shell 脚本
```bash
#!/bin/bash
for file in *.wav; do
    echo "Processing $file..."
    audio_normalizer -q -m -12 "$file" "normalized_$file"
done
echo "All files processed."
```

## 常见问题

### Q: 如何选择合适的目标电平？
**A**: 
- **流媒体**: -12 到 -14 dB
- **CD 制作**: -3 到 -6 dB  
- **广播**: -18 到 -23 dB
- **后期制作**: -18 到 -20 dB

### Q: 为什么输出电平不完全准确？
**A**: 这是正常现象，通常误差在 ±0.1 dB 内，由于：
- 浮点数精度限制
- 削波保护机制
- 量化噪声

### Q: 如何处理立体声文件？
**A**: 程序自动处理多声道文件，峰值检测会考虑所有声道的最大值。

### Q: 可以处理大文件吗？
**A**: 程序会将整个文件加载到内存中处理，确保有足够的 RAM。对于非常大的文件，建议分段处理。

## 注意事项

1. **备份原始文件**: 标准化是不可逆过程
2. **削波保护**: 程序会自动限制输出防止削波，但可能影响动态范围
3. **格式兼容性**: 某些格式可能不支持所有采样率和位深度组合
4. **内存使用**: 大文件会消耗相应的内存空间
