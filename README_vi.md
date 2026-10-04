# ⚡ Riot Account Switcher (LMHT & VALORANT)

<p align="center">
  <img src="web/assets/icon_256.png" width="128" height="128" alt="Riot Account Switcher Logo">
  <br>
  <strong>Trình quản lý & chuyển đổi nhanh tài khoản Riot Games siêu tốc cho Windows</strong>
  <br>
  <em>Dành cho game thủ Liên Minh Huyền Thoại (League of Legends) và VALORANT</em>
</p>

<p align="center">
  <a href="README.md">English</a> | <strong>Tiếng Việt</strong>
</p>

<p align="center">
  <a href="https://github.com/vietanh210304/riot-account-switcher/releases/latest">
    <img src="https://img.shields.io/github/v/release/vietanh210304/riot-account-switcher?style=flat-square&color=c8aa6e" alt="Bản phát hành mới nhất">
  </a>
  <img src="https://img.shields.io/badge/N%E1%BB%81n%20t%E1%BA%A3ng-Windows%2010%20%7C%2011-0ac8b9?style=flat-square" alt="Nền tảng">
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/Gi%E1%BA%A5y%20ph%C3%A9p-MIT-ff4655?style=flat-square" alt="Giấy phép">
  </a>
</p>

---

## 🌟 Tính năng nổi bật

- ⚡ **Hoán đổi tài khoản 1 chạm:** Chuyển đổi giữa các tài khoản Riot Games nhanh chóng chỉ trong vài giây mà không cần nhập lại mật khẩu hay mã xác thực OTP.
- 🎯 **Khởi chạy game trực tiếp qua Lockfile REST API:** Gọi trực tiếp REST API nội bộ của Riot Client (`POST /product-launcher/v1/products/.../patchlines/live`), mở thẳng LMHT hoặc VALORANT mà không sợ kẹt tiến trình chạy nền.
- 🌐 **Hỗ trợ song ngữ (Tiếng Việt & English):** Chuyển đổi ngôn ngữ tức thì chỉ với một cú nhấp chuột trên thanh tiêu đề ứng dụng.
- 🖱️ **Menu chuột phải tiện lợi (Right-click Context Menu):**
  - Chỉnh sửa tên hiển thị Riot ID & tagline (`#tag`).
  - Thêm ghi chú riêng cho từng tài khoản.
  - Chọn trong danh sách **24 Avatar đại diện chính thức** (12 tướng LMHT + 12 đặc vụ VALORANT, chạy offline 100%).
  - Sao chép nhanh Riot ID vào clipboard.
  - Xóa tài khoản với hộp thoại xác nhận In-App an toàn.
- 🛡️ **Bảo mật tối đa & Lưu trữ cục bộ:** Toàn bộ dữ liệu phiên đăng nhập lưu trữ 100% trên máy tính cá nhân của bạn. Không gửi bất kỳ dữ liệu nào ra bên ngoài.

---

## 📥 Tải về bản phát hành mới nhất

👉 Tải các gói cài đặt trực tiếp tại trang **[Releases](https://github.com/vietanh210304/riot-account-switcher/releases/latest)**:

- 📦 **Bản cài đặt (`RiotAccountSwitcher-Setup.exe`):** Trình cài đặt chuẩn Windows, tự động tạo lối tắt trên Desktop, Start Menu và hỗ trợ gỡ cài đặt tiện lợi trong Settings/Control Panel.
- 🚀 **Bản Portable (`RiotAccountSwitcher-Portable.exe`):** File thực thi đơn lẻ không cần cài đặt, nhấp đúp là sử dụng ngay ở bất kỳ thư mục nào!

---

## 🛠️ Hướng dẫn phát triển & Đóng gói từ nguồn

### Yêu cầu môi trường:
- Windows 10 / 11 (64-bit)
- Python 3.10+
- Microsoft Edge WebView2 Runtime (đã tích hợp sẵn trên Windows 10/11)

### Cài đặt thư viện:
```bash
git clone https://github.com/vietanh210304/riot-account-switcher.git
cd riot-account-switcher
pip install pywebview pyinstaller psutil pillow
```

### Chạy ứng dụng chế độ phát triển:
```bash
python app.py
```

### Đóng gói file `.exe` độc lập:
```bash
pyinstaller --noconsole --onefile --icon "assets/icon.ico" --name "RiotAccountSwitcher" --add-data "web;web" --add-data "assets;assets" --add-data "backend.py;." app.py -y
```
File thực thi hoàn chỉnh sẽ được tạo tại `dist/RiotAccountSwitcher.exe`.

---

## ⚖️ Giấy phép & Tuyên bố miễn trừ trách nhiệm

Dự án được phát hành dưới giấy phép [MIT License](LICENSE).

*Tuyên bố miễn trừ trách nhiệm:* Riot Account Switcher là một tiện ích phi chính thức và không liên kết, tài trợ hay chứng thực bởi Riot Games, Inc. Riot Games, League of Legends, và VALORANT là các nhãn hiệu hoặc nhãn hiệu đã đăng ký của Riot Games, Inc.
