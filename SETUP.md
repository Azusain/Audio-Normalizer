# 快速设置指南

## 前置依赖安装

### 1. 安装 vcpkg 包管理器

```bash
# 克隆 vcpkg
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg

# 运行 bootstrap 脚本
C:\vcpkg\bootstrap-vcpkg.bat

# 集成到 Visual Studio (可选)
C:\vcpkg\vcpkg integrate install
```

### 2. 安装 libsndfile 库

```bash
# 安装 libsndfile
C:\vcpkg\vcpkg install libsndfile

# 如果你需要 64 位版本
C:\vcpkg\vcpkg install libsndfile:x64-windows
```

### 3. 设置环境变量 (推荐)

将以下环境变量添加到系统环境变量中：
```
VCPKG_ROOT=C:\vcpkg
```

## 构建项目

### 方法一：使用批处理脚本 (推荐)

直接双击运行 `build.bat` 文件，或在命令提示符中运行：

```cmd
build.bat
```

### 方法二：手动构建

```bash
# 创建构建目录
mkdir build
cd build

# 配置项目
cmake .. -DCMAKE_TOOLCHAIN_FILE=C:/vcpkg/scripts/buildsystems/vcpkg.cmake

# 构建项目
cmake --build . --config Release
```

## 测试程序

构建成功后，可执行文件位于：
- `build/Release/audio_normalizer.exe` (Visual Studio)
- 或 `build/audio_normalizer.exe` (MinGW 等其他编译器)

测试命令：
```bash
# 显示帮助信息
audio_normalizer.exe --help

# 示例用法（需要有音频文件）
audio_normalizer.exe -m -12 input.wav output.wav
```

## 常见问题

### Q: CMake 找不到 libsndfile
**A**: 确保已通过 vcpkg 安装了 libsndfile，并正确设置了 CMAKE_TOOLCHAIN_FILE。

### Q: 编译器找不到
**A**: 安装 Visual Studio Community（包含 MSVC 编译器）或 MinGW-w64。

### Q: vcpkg 集成失败
**A**: 以管理员权限运行命令提示符，然后重新运行 vcpkg 命令。

### Q: 运行时找不到 DLL
**A**: 确保 vcpkg 安装的库版本与编译时使用的版本匹配（x86 vs x64）。

## 支持的音频格式

- **WAV**: 最常用的无压缩格式
- **FLAC**: 无损压缩格式  
- **OGG**: Ogg Vorbis 有损压缩
- **AU**: Sun/NeXT 音频格式
- **AIFF**: Apple 音频交换格式

**注意**: MP3 支持取决于你的 libsndfile 编译配置。如果需要 MP3 支持，可能需要额外安装编解码器。
