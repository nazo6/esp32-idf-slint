use slint_renderer_software_custom::{PremultipliedRgbaColor, TargetPixel};

/// SPIディスプレイ送信用に、メモリ上にあらかじめビッグエンディアン
/// （上位バイト -> 下位バイト）の順で配置されるRGB565ピクセル。
#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct BigEndianRgb565Pixel(pub u8, pub u8);

impl BigEndianRgb565Pixel {
    /// 現在のピクセル値を8ビットのRGB (R, G, B) にデコードするヘルパー関数。
    /// アルファブレンドの際に現在の背景色を取得するために使用します。
    #[inline(always)]
    fn to_rgb(self) -> (u8, u8, u8) {
        let high = self.0;
        let low = self.1;

        let r5 = high >> 3;
        let g6 = ((high & 0b111) << 3) | (low >> 5);
        let b5 = low & 0b11111;

        // 5bit/6bitの値を8bitにスケールアップして返す
        (
            (r5 << 3) | (r5 >> 2),
            (g6 << 2) | (g6 >> 4),
            (b5 << 3) | (b5 >> 2),
        )
    }
}

impl TargetPixel for BigEndianRgb565Pixel {
    #[inline(always)]
    fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        // 8bitの色情報を5bit, 6bit, 5bitに切り詰める
        let r5 = red >> 3;
        let g6 = green >> 2;
        let b5 = blue >> 3;

        // 16bitのRGB565を構成し、ビッグエンディアンのバイト配列として格納
        let high = (r5 << 3) | (g6 >> 3);
        let low = (g6 << 5) | b5;

        Self(high, low)
    }

    fn blend(&mut self, color: PremultipliedRgbaColor) {
        if color.alpha == 0 {
            // 完全透過の場合は何もしない
            return;
        }
        if color.alpha == 255 {
            // 完全不透明の場合は上書き
            *self = Self::from_rgb(color.red, color.green, color.blue);
            return;
        }

        // 半透明の場合は背景とブレンドする
        let (bg_r, bg_g, bg_b) = self.to_rgb();

        // Premultiplied Alphaによるブレンド計算: result = fg + bg * (1 - alpha)
        // (colorのRGB値にはすでにアルファ値が乗算されています)
        let inv_alpha = (255 - color.alpha) as u32;

        // ESP32-S3は32bit演算が速いため、浮動小数点ではなく整数演算で処理します
        let r = (color.red as u32 * 255 + bg_r as u32 * inv_alpha) / 255;
        let g = (color.green as u32 * 255 + bg_g as u32 * inv_alpha) / 255;
        let b = (color.blue as u32 * 255 + bg_b as u32 * inv_alpha) / 255;

        *self = Self::from_rgb(r as u8, g as u8, b as u8);
    }

    fn background() -> Self {
        Self::from_rgb(100, 0, 0)
    }
}
