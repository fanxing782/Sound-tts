
# 利用系统 API 达成文本到语音的转换，并借助对不同声卡设备的选择来实现播放功能。
Achieve text-to-speech conversion by utilizing system APIs, and realize the playback function by means of selecting different sound card devices.

## 支持系统 (Support System)
- [x] Windows

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


### 3.播放 (Playback)

```rust
let value = SoundValue::create("文本","设备名称");

// play_count 播放次数 0 是一直循环播放

// play_interval 本次播放完成后，播放下一次之间的间隔时间

let value1 = SoundValue::new("文本",0,1,"设备名称");

//顺序播放

SoundTTs::speak(value);

SoundTTs::speak(value1);

//中断播放

SoundTTs::speak_interrupt(value1);

//停止播放

SoundTTs::stop("设备名称)");

```

