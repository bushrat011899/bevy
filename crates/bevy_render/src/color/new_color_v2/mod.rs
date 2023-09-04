use self::{rgb::RGB, alpha::Transparent, srgb::SRGB, adobe_rgb::ARGB};

mod alpha;
mod rgb;
mod srgb;
mod adobe_rgb;
mod oklab;
mod oklch;
mod xyz;
mod hsl;
mod hsv;
mod lch;
mod lab;

pub const fn rgb(red: f32, green: f32, blue: f32) -> RGB<f32> {
    RGB::new(red, green, blue)
}

pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Transparent<RGB<f32>, f32> {
    let color = RGB::new(red, green, blue);
    Transparent::new(color, alpha)
}

pub const fn rgb_u8(red: u8, green: u8, blue: u8) -> RGB<u8> {
    RGB::new(red, green, blue)
}

pub const fn rgba_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Transparent<RGB<u8>, u8> {
    let color = RGB::new(red, green, blue);
    Transparent::new(color, alpha)
}

pub const fn srgb(red: f32, green: f32, blue: f32) -> SRGB<f32> {
    SRGB::new(red, green, blue)
}

pub const fn srgba(red: f32, green: f32, blue: f32, alpha: f32) -> Transparent<SRGB<f32>, f32> {
    let color = SRGB::new(red, green, blue);
    Transparent::new(color, alpha)
}

pub const fn argb(red: f32, green: f32, blue: f32) -> ARGB<f32> {
    ARGB::new(red, green, blue)
}

pub const fn argba(red: f32, green: f32, blue: f32, alpha: f32) -> Transparent<ARGB<f32>, f32> {
    let color = ARGB::new(red, green, blue);
    Transparent::new(color, alpha)
}

pub trait ColorSpace: From<RGB<f32>> + Into<RGB<f32>> {
    fn as_transparent(self) -> Transparent<Self, f32> {
        Transparent::new(self, 1.0)
    }

    fn as_transparent_u8(self) -> Transparent<Self, u8> {
        Transparent::new(self, u8::MAX)
    }

    fn as_rgb(self) -> RGB<f32> {
        self.into()
    }

    fn as_rgba(self) -> Transparent<RGB<f32>, f32> {
        self.into().into()
    }
    
    fn as_rgb_u8(self) -> RGB<u8> {
        self.into().into()
    }
    
    fn as_rgba_u8(self) -> Transparent<RGB<u8>, u8> {
        Transparent::new(self.into().into(), u8::MAX)
    }
    
    fn as_srgb(self) -> SRGB<f32> {
        self.into().into()
    }
    
    fn as_srgba(self) -> Transparent<SRGB<f32>, f32> {
        Transparent::new(self.into().into(), 1.0)
    }
}

impl<C: From<RGB<f32>> + Into<RGB<f32>>> ColorSpace for C { }

pub trait Mix: Sized {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self;

    fn mix_in<C: Mix>(self, target: impl Into<Self>, t: f32) -> C
    where
        Self: Into<C>,
    {
        let start = self.into();
        let target = target.into().into();

        start.mix(target, t)
    }
}

impl Mix for f32 {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target = target.into();

        self * (1. - t) + target * t
    }
}

impl Mix for u8 {
    fn mix(self, target: impl Into<Self>, t: f32) -> Self {
        debug_assert!(0. <= t && t <= 1.);

        let target: Self = target.into();

        let result = (self as f32) * (1. - t) + (target as f32) * t;

        result.clamp(0., Self::MAX as f32).floor() as Self
    }
}

impl From<Transparent<RGB<f32>, f32>> for wgpu::Color {
    fn from(color: Transparent<RGB<f32>, f32>) -> Self {
        Self {
            r: color.r() as f64,
            g: color.g() as f64,
            b: color.b() as f64,
            a: color.a() as f64,
        }
    }
}

impl encase::ShaderType for Transparent<RGB<f32>, f32> {
    type ExtraMetadata = ();

    const METADATA: encase::private::Metadata<Self::ExtraMetadata> = {
        let size =
            encase::private::SizeValue::from(<f32 as encase::private::ShaderSize>::SHADER_SIZE)
                .mul(4);
        let alignment = encase::private::AlignmentValue::from_next_power_of_two_size(size);

        encase::private::Metadata {
            alignment,
            has_uniform_min_alignment: false,
            min_size: size,
            extra: (),
        }
    };

    const UNIFORM_COMPAT_ASSERT: fn() = || {};
}

impl encase::private::WriteInto for Transparent<RGB<f32>, f32> {
    fn write_into<B: encase::private::BufferMut>(&self, writer: &mut encase::private::Writer<B>) {
        encase::private::WriteInto::write_into(&self.r(), writer);
        encase::private::WriteInto::write_into(&self.g(), writer);
        encase::private::WriteInto::write_into(&self.b(), writer);
        encase::private::WriteInto::write_into(&self.a(), writer);
    }
}

impl encase::private::ReadFrom for Transparent<RGB<f32>, f32> {
    fn read_from<B: encase::private::BufferRef>(
        &mut self,
        reader: &mut encase::private::Reader<B>,
    ) {
        let mut buffer = [0.0f32; 4];
        for el in &mut buffer {
            encase::private::ReadFrom::read_from(el, reader);
        }

        let color = RGB::new(buffer[0], buffer[1], buffer[2]);

        *self = Self::new(color, buffer[3])
    }
}

impl encase::private::CreateFrom for Transparent<RGB<f32>, f32> {
    fn create_from<B>(reader: &mut encase::private::Reader<B>) -> Self
    where
        B: encase::private::BufferRef,
    {
        // These are intentionally not inlined in the constructor to make this
        // resilient to internal Color refactors / implicit type changes.
        let red: f32 = encase::private::CreateFrom::create_from(reader);
        let green: f32 = encase::private::CreateFrom::create_from(reader);
        let blue: f32 = encase::private::CreateFrom::create_from(reader);
        let alpha: f32 = encase::private::CreateFrom::create_from(reader);

        let color = RGB::new(red, green, blue);

        Self::new(color, alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::{rgb::RGB, Mix, alpha::Transparent};

    #[test]
    fn example() {
        let red = Transparent::new(RGB::<f32>::new(1., 0., 0.), 1.0);
        let blue = Transparent::new(RGB::<f32>::new(0., 0., 1.), 1.0);

        let purple = red.mix(blue, 0.5);

        assert_eq!(purple.r(), 0.5);
        assert_eq!(purple.g(), 0.0);
        assert_eq!(purple.b(), 0.5);
        assert_eq!(purple.a(), 1.0);
    }
}