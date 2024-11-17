
# 利用系统 API 达成文本到语音的转换，并借助对不同声卡设备的选择来实现播放功能。
Achieve text-to-speech conversion by utilizing system APIs, and realize the playback function by means of selecting different sound card devices.

## 支持系统 (Support System)
- [x] Windows

- [ ] Linux 



## 例子 (Example)
### 1.初始化 Initialization
在程序首次启动之前，需调用一次初始化函数，以完成相关设置。示例代码如下：

Before the program is launched for the first time, the initialization function needs to be called once to complete the relevant settings. The sample code is as follows:
```rust
use Sound_tts::SoundTTs;
SoundTTs::init();
```

### 2.获取设备列表 (Get device list)
```rust
let devices: Vec<String> = SoundTTs::get_devices();
for x in devices {
    println!("{}", x);
}
```


### 3.播放 Playback

#### 顺序播放
```rust
SoundTTs::speak("test", "device_name");
SoundTTs::speak("test1", "device_name");
```
#### 中断播放
```rust
SoundTTs::speak_interrupt("test2", "device_name");
```
#### 停止播放
```rust
SoundTTs::stop("设备名称)");
```


