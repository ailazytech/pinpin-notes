# PinPin Notes

<img src="assets/logo/pinpin-256.png" width="112" align="right" alt="PinPin Notes">

[English](README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · **日本語** · [한국어](README.ko.md) · [العربية](README.ar.md)

[![GitHub stars](https://img.shields.io/github/stars/ailazytech/pinpin-notes?style=flat&logo=github&label=Stars)](https://github.com/ailazytech/pinpin-notes/stargazers) [![GitHub watchers](https://img.shields.io/github/watchers/ailazytech/pinpin-notes?style=flat&logo=github&label=Watchers)](https://github.com/ailazytech/pinpin-notes/watchers) [![GitHub followers](https://img.shields.io/github/followers/ailazytech?style=flat&logo=github&label=Followers)](https://github.com/ailazytech) [![Latest release](https://img.shields.io/github/v/release/ailazytech/pinpin-notes?style=flat&label=Release)](https://pinpin.fast01.net) [![CI](https://github.com/ailazytech/pinpin-notes/actions/workflows/ci.yml/badge.svg)](https://github.com/ailazytech/pinpin-notes/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE) [![Windows 10 | 11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?logo=windows&logoColor=white)](https://pinpin.fast01.net) [![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)

> **デスクトップの小さなメモが、いつもそばに。**

Windows デスクトップの付箋アプリ。画鋲、マスキングテープ、ステッカーを自由に貼れて、手書き、透明スローガン、ポモドーロタイマーも。普段はほとんど電力を使いません。

- `.exe` ひとつ（約 11 MB）、インストール不要、Windows 10 / 11
- アイドル時 CPU 0%、メモリ約 15 MB（待機中のタイマーは一切なし。画面が変わるときだけ再描画）
- 6 言語：English、繁體中文、简体中文、日本語、한국어、العربية（右から左）
- オープンソース（MIT）、付箋は自分のパソコンにだけ保存。通信は 1 日 1 回まで、匿名の利用統計はオフにできます（[プライバシー](docs/PRIVACY.md)）

🌐 公式サイト：**[pinpin.fast01.net](https://pinpin.fast01.net)**　📦 ダウンロード：[公式サイト](https://pinpin.fast01.net)（準備中）　📝 [更新履歴](CHANGELOG.md)

## 機能

- **付箋**：Windows 起動時にデスクトップへ戻ります。ドラッグで移動、ほかの付箋や画面の端に近づくとそっと吸着して揃います（Alt を押しながらで吸着なし、矢印キーで 1px ずつ微調整）
- **線で消す**：行にマウスを乗せると左に丸が出ます。クリックで赤ペンの取り消し線（アニメーション付き）
- **画鋲・クリップ・マスキングテープ・ステッカー**：「ステッカーショップ」から付箋やデスクトップへドラッグ。ステッカー 148 枚（4 倍解像度で拡大してもくっきり）を、日常、開運、世界の縁起物、観葉植物、春聯、猫と犬、スクーター、台湾の 8 つに分類。クリックするとハンドルが出て、右上で回転、右下で拡大縮小。マスキングテープは両端をドラッグして伸ばしたり向きを変えたりできます。選択していない画鋲・クリップ・テープをドラッグすると付箋ごと動きます
- **自作ステッカー**：ショップに自分のグループを作り、画像をドロップ、Ctrl+V、または「＋」。背景を自動で切り抜き、丸みのある白い縁を付けます。元のサイズ（最大 4096px）のまま JPEG XR で省スペース保存。内蔵ステッカーもグループに入れられます
- **オーダーメイド**：ショップの最後のカードから公式サイトのオーダーページへ（ペットのステッカー、タイマーのキャラクター、テーマセット）
- **手書き**：ペン、蛍光ペン、チョーク、クレヨン、8 色、3 段階の太さ、消しゴム、Ctrl+Z で元に戻す。**透明な紙**ならデスクトップに浮かぶ手書きスローガンに
- **文字のスタイル**：色、フォント、太字、サイズ（最大 6 倍）。トレイの「既定の文字スタイル」で全体、右クリックの「文字スタイル（この付箋）」で 1 枚だけ変更
- **ポモドーロタイマー**：トマト、砂時計、LED 時計、目覚まし時計、時計を抱えた猫、時計を抱えた柴犬。クリックで開始／一時停止、ホイールで分を設定。時間になると鳴ります（内蔵チャイム、Windows のアラーム音、自分の .wav、または無音）。止まっている間は電力を使いません
- **紙**：定番 8 色＋モランディ 8 色＋透明。付箋、なめらか、方眼、罫線、ドット方眼、和紙、クラフト繊維
- **画像**：画像ファイルをドロップするか付箋の上で Ctrl+V でポラロイド風に
- **パスワード**：付箋ごとに別のパスワードを設定（PBKDF2 + AES-256-GCM）。普段は何気ないダミーの ToDo を表示し、パスワードを入れると文字が乱れて本当の内容に戻ります。閉じたり再起動したりすると自動で再ロック
- **スター付きノート**：Vibotype の選択ツールバーと組み合わせると、どのアプリでも文字を選んで ★ を押すだけで集められます
- **JSON でエクスポート／インポート**：別のパソコンへ移したり手動で同期したり（画像、手書き、自作ステッカーとグループもまとめて。暗号化した内容は暗号化のまま）
- **デスクトップの右クリック**「新しい付箋」「新しいタイマー」と **Windows 起動時に開始**、どちらも初期設定はオフ、トレイメニューでオンにできます
- **オンライン更新**：新しいバージョンを静かに確認（トレイメニューにだけ表示）。重要な更新は自動でダウンロードして次回起動時に適用。新しいステッカーパックやショップのバナーも届きます。すべてオフにできます（[docs/UPDATES.md](docs/UPDATES.md)）
- **言語**：再ビルドなしで自分の言語を追加できます（[locales/README.md](locales/README.md)）

## 使い方

- トレイアイコン：**左クリック**で新しい付箋、**右クリック**でメニュー（手書き、透明スローガン、タイマーもここから）
- 手書き：付箋右上のペン、または右クリックの「手書き」。Esc／Enter で終了、E で消しゴム切り替え
- タイマー：スペースで開始／一時停止、R でリセット。右クリックでスタイル、時間、音を変更
- 付箋の上で：ダブルクリックで編集、右クリックでメニュー、Delete で削除（トレイから元に戻せます）、Ctrl+L でロック
- データは `%APPDATA%\PinPin\` に保存。形式は [docs/DATA.md](docs/DATA.md)

## 開発者向け

ビルド方法、開発用オプション、プロジェクト構成、ほかのアプリから PinPin を呼ぶ方法は[英語版 README](README.md#building) をご覧ください。Issue と Pull Request を歓迎します（[CONTRIBUTING.md](CONTRIBUTING.md)）。

自分のペットのステッカーやタイマーのキャラクターが欲しい方は[公式サイトのオーダーページ](https://pinpin.fast01.net/#/custom)、または ailazytech@gmail.com まで。

## ライセンス

[MIT](LICENSE)。内蔵の画鋲、テープ、ステッカー、タイマーの画像は AI 画像生成のあと加工したもので、同じく MIT で公開しています。
