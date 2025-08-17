# Audio Normalizer

一个用于音频峰值电平标准化的命令行工具，使用 C++ 开发，基于 libsndfile 库。

## 功能特性

- **峰值检测**：分析音频文件的峰值电平
- **标准化处理**：将音频峰值调整到指定的 dB 电平
- **多格式支持**：支持 WAV、FLAC、OGG、MP3 等多种音频格式
- **防削波保护**：自动限制输出信号防止削波失真
- **详细输出**：可选的详细处理信息显示

## 构建依赖

- CMake 3.20+
- C++17 编译器
- vcpkg 包管理器
- libsndfile (通过 vcpkg 安装)

## 构建步骤

1. 克隆或下载项目源码
2. 安装 vcpkg 并设置环境变量
3. 使用 CMake 构建项目：

```bash
# 创建构建目录
mkdir build
cd build

# 配置项目 (Windows)
cmake .. -DCMAKE_TOOLCHAIN_FILE=C:/vcpkg/scripts/buildsystems/vcpkg.cmake

# 或者 (Linux/macOS)
cmake .. -DCMAKE_TOOLCHAIN_FILE=/path/to/vcpkg/scripts/buildsystems/vcpkg.cmake

# 构建
cmake --build . --config Release
```

## 使用方法

### 基本语法
```
audio_normalizer [options] input_file output_file
```

### 命令行选项

- `-m, --max-peak <peak_db>`: 设置目标峰值电平 (dB)，默认 -12.0
- `-v, --verbose`: 启用详细输出（调试级别日志）
- `-q, --quiet`: 启用安静模式（仅错误级别日志）
- `-h, --help`: 显示帮助信息
- `--peak`: 仅显示输入文件的峰值电平（不进行标准化）

### 使用示例

1. **标准化到 -12 dB 峰值（使用默认值）**：
   ```bash
   audio_normalizer input.wav output.wav
   ```

2. **标准化到 -6 dB 峰值**：
   ```bash
   audio_normalizer -m -6 input.flac output.flac
   ```

3. **标准化并显示详细调试信息**：
   ```bash
   audio_normalizer -m -12 -v input.wav output.wav
   ```

4. **仅查看文件的峰值电平**：
   ```bash
   audio_normalizer --peak input.mp3
   ```

5. **安静模式处理（仅显示错误）**：
   ```bash
   audio_normalizer -q -m -6 input.wav output.wav
   ```

## 音频标准化原理

音频标准化是将音频信号的最大峰值调整到指定电平的过程：

1. **峰值检测**：扫描整个音频文件，找到最大的绝对值样本
2. **增益计算**：根据当前峰值和目标峰值计算所需的增益
3. **信号处理**：对所有样本应用计算出的增益
4. **削波保护**：确保处理后的信号不超出 [-1.0, 1.0] 范围

### 公式

- **线性值转 dB**：`dB = 20 * log10(linear)`
- **dB 转线性值**：`linear = 10^(dB/20)`
- **增益计算**：`gain_dB = target_dB - current_peak_dB`

## 技术实现

- **音频 I/O**：使用 libsndfile 库处理多种音频格式
- **命令行解析**：使用 cxxopts 库提供健壮的参数处理
- **日志系统**：使用 spdlog 库提供结构化和彩色日志输出
- **数值处理**：64位双精度浮点运算确保精度
- **内存管理**：智能指针和 RAII 模式确保资源安全
- **错误处理**：完整的错误检查和用户友好的错误信息

## 支持的音频格式

通过 libsndfile 支持的格式包括：
- WAV (PCM, Float)
- FLAC
- OGG Vorbis
- AIFF
- AU
- 以及其他 libsndfile 支持的格式

## 注意事项

- 输入和输出文件格式由文件扩展名自动识别
- 标准化不会改变音频的动态范围，只是整体音量调整
- 对于已经接近 0 dB 的音频，向上标准化可能导致削波
- 程序会自动应用削波保护，但可能影响音质

## 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。
