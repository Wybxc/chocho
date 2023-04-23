# chocho

渐进式 QQ 机器人快速开发框架，基于 [ricq](https://github.com/lz1998/ricq)。

## Examples

```rust
use chocho::prelude::*;

use async_trait::async_trait;
use chocho::ricq::handler::PartlyHandler;

struct Handler;
#[async_trait]
impl PartlyHandler for Handler {
    async fn handle_login(&self, uin: i64) {
        tracing::info!("登录成功: {}", uin);
    }
}

#[chocho::main(handler = Handler)]
async fn main(client: RQClient) -> anyhow::Result<()> {
    let account_info = client.account_info.read().await;
    tracing::info!("{:?}", account_info);

    client.group(12345678).send("Hello, world!").await?;
}
```

## 已完成功能/开发计划

<details>
<summary>点击展开</summary>

### 模块

- [x] chocho: 统一入口点
- [x] chocho_login: 用户登录
- [x] chocho_client: 客户端操作接口
- [ ] chocho_event: 事件处理
- [x] chocho_msg: 消息处理
- [x] chocho_macros: 过程宏支持

### 登录

- [x] 账号密码登录
- [x] 二维码登录
- [x] 验证码提交
- [x] 设备锁验证
- [x] 错误信息解析

### 消息类型

- [x] 文本
- [x] 表情
- [x] At
- [x] 回复
- [x] 匿名
- [x] 骰子
- [x] 石头剪刀布
- [x] 图片
- [x] 语音
- [x] 长消息(仅支持群聊发送)
- [ ] 合并转发(仅支持群聊发送)
- [x] 链接分享
- [ ] 小程序(暂只支持RAW)
- [ ] 短视频
- [ ] 群文件(上传与接收信息)

### 事件

> 支持使用 ricq 的事件处理（`Handler`）处理事件，以下为 `chocho_event` 的实现进度

- [ ] 群消息
- [ ] 好友消息
- [ ] 新好友请求
- [ ] 收到其他用户进群请求
- [ ] 新好友
- [ ] 群禁言
- [ ] 好友消息撤回
- [ ] 群消息撤回
- [ ] 收到邀请进群请求
- [ ] 群名称变更
- [ ] 好友删除
- [ ] 群成员权限变更
- [ ] 新成员进群/退群
- [ ] 登录号加群
- [ ] 临时会话消息
- [ ] 群解散
- [ ] 登录号退群(包含T出)
- [ ] 客户端离线
- [ ] 群提示 (戳一戳/运气王等)

### 主动操作

#### 通用

- [ ] 修改昵称
- [ ] 设置在线状态
- [ ] 修改个人资料
- [ ] 修改个性签名

#### 好友操作

- [x] 发送好友消息
- [ ] 获取好友列表/分组
- [ ] 添加/删除/重命名好友分组
- [x] 获取好友个性签名
- [x] 戳一戳好友
- [x] 发送好友语音
- [x] 下载好友语音
- [x] 好友链接分享
- [x] 好友音乐分享
- [x] 撤回好友消息
- [ ] 处理好友请求
- [x] 删除好友
- [ ] 获取陌生人信息


#### 群操作

> 为防止滥用，将不支持主动邀请新成员进群

- [x] 发送群消息
- [ ] 获取群列表
- [x] 获取群成员列表
- [x] 获取群管理员列表
- [x] 群成员禁言/解除禁言
- [x] 踢出群成员
- [x] 戳一戳群友
- [x] 发送群语音
- [ ] 下载群语音
- [x] 群链接分享
- [x] 群音乐分享
- [x] 群匿名消息
- [x] 群打卡
- [x] 设置/取消群管理员
- [x] 设置群公告
- [x] 设置群名称
- [ ] 全员禁言
- [x] 获取群@全体剩余次数
- [x] 修改群成员头衔
- [x] 获取群成员信息。
- [ ] 设置群精华消息
- [x] 发送临时会话消息
- [x] 修改群成员名片
- [x] 撤回群消息
- [ ] 处理被邀请加群请求
- [ ] 处理加群请求
- [ ] 获取群荣誉 (龙王/群聊火焰等)
- [ ] 获取群文件下载链接
- [ ] ~~群成员邀请~~

#### 其他

- [ ] 翻译
- [ ] OCR

</details>
