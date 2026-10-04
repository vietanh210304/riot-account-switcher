# ⚡ Riot Account Switcher (LMHT & VALORANT)

<p align="center">
  <img src="web/assets/icon_256.png" width="128" height="128" alt="Riot Account Switcher Logo">
  <br>
  <strong>Trình quản lý & chuyển đổi nhanh tài khoản Riot Games siêu tốc cho Windows</strong>
  <br>
  <em>Dành cho game thủ Liên Minh Huyền Thoại (League of Legends) và VALORANT</em>
</p>

---

## 🌟 Tính năng nổi bật

- ⚡ **Hoán đổi tài khoản 1 chạm:** Chuyển đổi giữa các tài khoản Riot Games nhanh chóng mà không cần nhập lại mật khẩu hay OTP.
- 🎯 **Khởi chạy game trực tiếp:** Gọi REST API nội bộ thông qua Lockfile của Riot Client, mở thẳng Liên Minh Huyền Thoại hoặc VALORANT mà không sợ kẹt tiến trình chạy nền.
- 🎨 **Giao diện Hextech Arcana tinh tế:** Tông màu Navy & Vàng kim sang trọng, bo viền mềm mại, font chữ tiếng Việt `Be Vietnam Pro` sắc nét, không bị lỗi font hay đè chữ.
- 🖱️ **Menu chuột phải tiện lợi (Right-click Context Menu):**
  - Đổi tên hiển thị & Tagline (`#tag`).
  - Ghi chú tài khoản.
  - Chọn 24 Avatar đại diện chính thức từ các tướng LMHT và đặc vụ VALORANT (hoạt động offline 100%).
  - Sao chép nhanh Riot ID.
  - Xóa tài khoản với hộp thoại xác nhận In-App an toàn.
- 🛡️ **Bảo mật tối đa:** Toàn bộ token phiên được lưu trữ cục bộ trên máy tính của bạn trong thư mục dữ liệu cá nhân, không gửi bất kỳ thông tin nào ra máy chủ bên ngoài.
- 📦 **Đóng gói Native Windows:** File `.exe` duy nhất (Single-file Portable ~20MB), tích hợp sẵn icon cho Desktop, Taskbar và Titlebar, không yêu cầu cài đặt Python hay Node.js.

---

## 📥 Tải về bản phát hành mới nhất

👉 Tải file thực thi trực tiếp tại trang **[Releases](https://github.com/vietanh210304/riot-account-switcher/releases/latest)**:
- Tải file `RiotAccountSwitcher.exe`.
- Đặt vào thư mục bất kỳ trên máy và chạy trực tiếp!

---

## 🚀 Hướng dẫn phát triển & Đóng gói từ nguồn

### Yêu cầu môi trường:
- Windows 10 / 11 64-bit
- Python 3.10+
- Microsoft Edge WebView2 Runtime (đã tích hợp sẵn trên Windows 10/11)

### Cài đặt thư viện:
```bash
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

---

## ⚖️ Giấy phép (License)
Dự án được phát hành dưới giấy phép [MIT License](LICENSE).
Tất cả hình ảnh, logo và thương hiệu Riot Games, League of Legends, VALORANT thuộc bản quyền của Riot Games, Inc.
