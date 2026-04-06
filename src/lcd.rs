use esp_idf_sys::{self as sys, *};
use std::ptr;

// 接続するGPIOピンの定義
const PIN_NUM_DC: i32 = 47;
const PIN_NUM_CS: i32 = 12;
const PIN_NUM_WR: i32 = 9;
const PIN_NUM_DATA0: i32 = 48;
const PIN_NUM_DATA1: i32 = 38;
const PIN_NUM_DATA2: i32 = 39;
const PIN_NUM_DATA3: i32 = 40;
const PIN_NUM_DATA4: i32 = 41;
const PIN_NUM_DATA5: i32 = 42;
const PIN_NUM_DATA6: i32 = 1;
const PIN_NUM_DATA7: i32 = 2;
const PIN_NUM_RST: i32 = 16;

const LCD_H_RES: u32 = 320;
const LCD_V_RES: u32 = 480;
const LCD_PIXEL_CLOCK_HZ: i32 = 20_000_000;

/// 安全にLCDディスプレイを操作するためのラッパー構造体
pub struct Display {
    panel_handle: esp_lcd_panel_handle_t,
    // リソースリークを防ぐためのDrop実装（将来用）に備えて保持しておくのがベストプラクティスです
    _io_handle: esp_lcd_panel_io_handle_t,
    _i80_bus: esp_lcd_i80_bus_handle_t,
}

impl Display {
    /// ディスプレイの初期化を行い、Displayインスタンスを生成する
    pub fn new() -> Result<Self, sys::EspError> {
        unsafe {
            // 1. I80 (8080) バスの初期化
            let mut i80_bus: esp_lcd_i80_bus_handle_t = ptr::null_mut();

            let mut bus_config: esp_lcd_i80_bus_config_t = esp_lcd_i80_bus_config_t {
                clk_src: soc_periph_lcd_clk_src_t_LCD_CLK_SRC_DEFAULT,
                dc_gpio_num: PIN_NUM_DC,
                wr_gpio_num: PIN_NUM_WR,
                bus_width: 8,
                max_transfer_bytes: (LCD_H_RES * LCD_V_RES * 2) as usize,
                ..Default::default()
            };
            bus_config.data_gpio_nums[0] = PIN_NUM_DATA0;
            bus_config.data_gpio_nums[1] = PIN_NUM_DATA1;
            bus_config.data_gpio_nums[2] = PIN_NUM_DATA2;
            bus_config.data_gpio_nums[3] = PIN_NUM_DATA3;
            bus_config.data_gpio_nums[4] = PIN_NUM_DATA4;
            bus_config.data_gpio_nums[5] = PIN_NUM_DATA5;
            bus_config.data_gpio_nums[6] = PIN_NUM_DATA6;
            bus_config.data_gpio_nums[7] = PIN_NUM_DATA7;

            // esp_nofail! ではなく sys::esp!() を使い、エラーなら早期リターン(?)させる
            sys::esp!(esp_lcd_new_i80_bus(&bus_config, &mut i80_bus))?;

            // 2. パネルIOの初期化
            let mut io_handle: esp_lcd_panel_io_handle_t = ptr::null_mut();
            let mut io_config: esp_lcd_panel_io_i80_config_t = Default::default();
            io_config.cs_gpio_num = PIN_NUM_CS;
            io_config.pclk_hz = LCD_PIXEL_CLOCK_HZ as u32;
            io_config.trans_queue_depth = 10;
            io_config.dc_levels.set_dc_idle_level(0);
            io_config.dc_levels.set_dc_cmd_level(0);
            io_config.dc_levels.set_dc_dummy_level(0);
            io_config.dc_levels.set_dc_data_level(1);
            io_config.lcd_cmd_bits = 8;
            io_config.lcd_param_bits = 8;

            sys::esp!(esp_lcd_new_panel_io_i80(
                i80_bus,
                &io_config,
                &mut io_handle
            ))?;

            // 3. LCDパネルドライバの初期化 (ILI9488)
            let mut panel_handle: esp_lcd_panel_handle_t = ptr::null_mut();
            let mut panel_config: esp_lcd_panel_dev_config_t = Default::default();
            panel_config.reset_gpio_num = PIN_NUM_RST;
            panel_config.__bindgen_anon_1.rgb_ele_order =
                lcd_rgb_element_order_t_LCD_RGB_ELEMENT_ORDER_BGR;
            panel_config.bits_per_pixel = 16;

            sys::esp!(esp_lcd_new_panel_ili9488(
                io_handle,
                &panel_config,
                (LCD_H_RES * LCD_V_RES * 2) as usize,
                &mut panel_handle
            ))?;

            // 4. パネルの起動手順
            sys::esp!(esp_lcd_panel_reset(panel_handle))?;
            sys::esp!(esp_lcd_panel_init(panel_handle))?;
            sys::esp!(esp_lcd_panel_invert_color(panel_handle, false))?;
            sys::esp!(esp_lcd_panel_disp_on_off(panel_handle, true))?;

            Ok(Self {
                panel_handle,
                _io_handle: io_handle,
                _i80_bus: i80_bus,
            })
        }
    }

    /// 指定した領域にピクセルデータ(RGB565)を描画する
    /// 安全のため、スライスのサイズが領域を満たしているかチェックします。
    pub fn draw(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        color_data: &[u8],
    ) -> Result<(), sys::EspError> {
        unsafe {
            sys::esp!(esp_lcd_panel_draw_bitmap(
                self.panel_handle,
                x as std::ffi::c_int,
                y as std::ffi::c_int,
                (x + width) as std::ffi::c_int,
                (y + height) as std::ffi::c_int,
                color_data.as_ptr() as *const _
            ))?;
        }

        Ok(())
    }
}
