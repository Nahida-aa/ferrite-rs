use std::fmt;

/// Java 对照: net.minecraft.network.codec.StreamCodec
pub trait StreamCodec<B, V> {
    fn encode(&self, buf: &mut B, value: &V);
    fn decode(&self, buf: &mut B) -> V;
}

// ---------------------------------------------------------------------------
// UnitCodec
// ---------------------------------------------------------------------------

/// Java 对照: `StreamCodec.unit(instance)`
pub struct UnitCodec<V> {
    instance: V,
}

impl<B, V: Clone + PartialEq + fmt::Debug> StreamCodec<B, V> for UnitCodec<V> {
    fn encode(&self, _buf: &mut B, value: &V) {
        assert!(
            *value == self.instance,
            "UnitCodec: encode mismatch"
        );
    }

    fn decode(&self, _buf: &mut B) -> V {
        self.instance.clone()
    }
}

pub fn unit<V: Clone>(instance: V) -> UnitCodec<V> {
    UnitCodec { instance }
}

// ---------------------------------------------------------------------------
// MapCodec
// ---------------------------------------------------------------------------

/// Java 对照: `StreamCodec.map(to, from)`
pub struct MapCodec<C, F, G> {
    inner: C,
    to: F,
    from: G,
}

impl<B, V, O, C, F, G> StreamCodec<B, O> for MapCodec<C, F, G>
where
    C: StreamCodec<B, V>,
    F: Fn(&V) -> O,
    G: Fn(&O) -> V,
{
    fn encode(&self, buf: &mut B, value: &O) {
        let inner = (self.from)(value);
        self.inner.encode(buf, &inner);
    }

    fn decode(&self, buf: &mut B) -> O {
        let inner = self.inner.decode(buf);
        (self.to)(&inner)
    }
}

pub fn map<B, V, O, C, F, G>(codec: C, to: F, from: G) -> MapCodec<C, F, G>
where
    C: StreamCodec<B, V>,
    F: Fn(&V) -> O,
    G: Fn(&O) -> V,
{
    MapCodec { inner: codec, to, from }
}

// ---------------------------------------------------------------------------
// Composite1Codec
// ---------------------------------------------------------------------------

/// Java 对照: `StreamCodec.composite(codec1, getter1, constructor)`
pub struct Composite1Codec<C, G, F> {
    codec: C,
    getter: G,
    constructor: F,
}

impl<B, V, T, C, G, F> StreamCodec<B, V> for Composite1Codec<C, G, F>
where
    C: StreamCodec<B, T>,
    G: Fn(&V) -> T,
    F: Fn(T) -> V,
{
    fn encode(&self, buf: &mut B, value: &V) {
        let f = (self.getter)(value);
        self.codec.encode(buf, &f);
    }

    fn decode(&self, buf: &mut B) -> V {
        let f = self.codec.decode(buf);
        (self.constructor)(f)
    }
}

pub fn composite1<B, V, T, C, G, F>(codec: C, getter: G, constructor: F) -> Composite1Codec<C, G, F>
where
    C: StreamCodec<B, T>,
    G: Fn(&V) -> T,
    F: Fn(T) -> V,
{
    Composite1Codec { codec, getter, constructor }
}

// ---------------------------------------------------------------------------
// Composite2Codec
// ---------------------------------------------------------------------------

/// Java 对照: `StreamCodec.composite(codec1, getter1, codec2, getter2, constructor)`
pub struct Composite2Codec<C1, C2, G1, G2, F> {
    codec1: C1,
    getter1: G1,
    codec2: C2,
    getter2: G2,
    constructor: F,
}

impl<B, V, T1, T2, C1, C2, G1, G2, F> StreamCodec<B, V> for Composite2Codec<C1, C2, G1, G2, F>
where
    C1: StreamCodec<B, T1>,
    C2: StreamCodec<B, T2>,
    G1: Fn(&V) -> T1,
    G2: Fn(&V) -> T2,
    F: Fn(T1, T2) -> V,
{
    fn encode(&self, buf: &mut B, value: &V) {
        let f1 = (self.getter1)(value);
        self.codec1.encode(buf, &f1);
        let f2 = (self.getter2)(value);
        self.codec2.encode(buf, &f2);
    }

    fn decode(&self, buf: &mut B) -> V {
        let f1 = self.codec1.decode(buf);
        let f2 = self.codec2.decode(buf);
        (self.constructor)(f1, f2)
    }
}

pub fn composite2<B, V, T1, T2, C1, C2, G1, G2, F>(
    codec1: C1,
    getter1: G1,
    codec2: C2,
    getter2: G2,
    constructor: F,
) -> Composite2Codec<C1, C2, G1, G2, F>
where
    C1: StreamCodec<B, T1>,
    C2: StreamCodec<B, T2>,
    G1: Fn(&V) -> T1,
    G2: Fn(&V) -> T2,
    F: Fn(T1, T2) -> V,
{
    Composite2Codec { codec1, getter1, codec2, getter2, constructor }
}

// ---------------------------------------------------------------------------
// Composite3Codec
// ---------------------------------------------------------------------------

/// Java 对照: `StreamCodec.composite(codec1, getter1, codec2, getter2, codec3, getter3, constructor)`
pub struct Composite3Codec<C1, C2, C3, G1, G2, G3, F> {
    codec1: C1,
    getter1: G1,
    codec2: C2,
    getter2: G2,
    codec3: C3,
    getter3: G3,
    constructor: F,
}

impl<B, V, T1, T2, T3, C1, C2, C3, G1, G2, G3, F> StreamCodec<B, V>
    for Composite3Codec<C1, C2, C3, G1, G2, G3, F>
where
    C1: StreamCodec<B, T1>,
    C2: StreamCodec<B, T2>,
    C3: StreamCodec<B, T3>,
    G1: Fn(&V) -> T1,
    G2: Fn(&V) -> T2,
    G3: Fn(&V) -> T3,
    F: Fn(T1, T2, T3) -> V,
{
    fn encode(&self, buf: &mut B, value: &V) {
        let f1 = (self.getter1)(value);
        self.codec1.encode(buf, &f1);
        let f2 = (self.getter2)(value);
        self.codec2.encode(buf, &f2);
        let f3 = (self.getter3)(value);
        self.codec3.encode(buf, &f3);
    }

    fn decode(&self, buf: &mut B) -> V {
        let f1 = self.codec1.decode(buf);
        let f2 = self.codec2.decode(buf);
        let f3 = self.codec3.decode(buf);
        (self.constructor)(f1, f2, f3)
    }
}

pub fn composite3<B, V, T1, T2, T3, C1, C2, C3, G1, G2, G3, F>(
    codec1: C1,
    getter1: G1,
    codec2: C2,
    getter2: G2,
    codec3: C3,
    getter3: G3,
    constructor: F,
) -> Composite3Codec<C1, C2, C3, G1, G2, G3, F>
where
    C1: StreamCodec<B, T1>,
    C2: StreamCodec<B, T2>,
    C3: StreamCodec<B, T3>,
    G1: Fn(&V) -> T1,
    G2: Fn(&V) -> T2,
    G3: Fn(&V) -> T3,
    F: Fn(T1, T2, T3) -> V,
{
    Composite3Codec { codec1, getter1, codec2, getter2, codec3, getter3, constructor }
}

// ---------------------------------------------------------------------------
// Composite4Codec
// ---------------------------------------------------------------------------

/// Java 对照: four-field composite
pub struct Composite4Codec<C1, C2, C3, C4, G1, G2, G3, G4, F> {
    codec1: C1,
    getter1: G1,
    codec2: C2,
    getter2: G2,
    codec3: C3,
    getter3: G3,
    codec4: C4,
    getter4: G4,
    constructor: F,
}

impl<B, V, T1, T2, T3, T4, C1, C2, C3, C4, G1, G2, G3, G4, F> StreamCodec<B, V>
    for Composite4Codec<C1, C2, C3, C4, G1, G2, G3, G4, F>
where
    C1: StreamCodec<B, T1>,
    C2: StreamCodec<B, T2>,
    C3: StreamCodec<B, T3>,
    C4: StreamCodec<B, T4>,
    G1: Fn(&V) -> T1,
    G2: Fn(&V) -> T2,
    G3: Fn(&V) -> T3,
    G4: Fn(&V) -> T4,
    F: Fn(T1, T2, T3, T4) -> V,
{
    fn encode(&self, buf: &mut B, value: &V) {
        let f1 = (self.getter1)(value);
        self.codec1.encode(buf, &f1);
        let f2 = (self.getter2)(value);
        self.codec2.encode(buf, &f2);
        let f3 = (self.getter3)(value);
        self.codec3.encode(buf, &f3);
        let f4 = (self.getter4)(value);
        self.codec4.encode(buf, &f4);
    }

    fn decode(&self, buf: &mut B) -> V {
        let f1 = self.codec1.decode(buf);
        let f2 = self.codec2.decode(buf);
        let f3 = self.codec3.decode(buf);
        let f4 = self.codec4.decode(buf);
        (self.constructor)(f1, f2, f3, f4)
    }
}

pub fn composite4<B, V, T1, T2, T3, T4, C1, C2, C3, C4, G1, G2, G3, G4, F>(
    codec1: C1,
    getter1: G1,
    codec2: C2,
    getter2: G2,
    codec3: C3,
    getter3: G3,
    codec4: C4,
    getter4: G4,
    constructor: F,
) -> Composite4Codec<C1, C2, C3, C4, G1, G2, G3, G4, F>
where
    C1: StreamCodec<B, T1>,
    C2: StreamCodec<B, T2>,
    C3: StreamCodec<B, T3>,
    C4: StreamCodec<B, T4>,
    G1: Fn(&V) -> T1,
    G2: Fn(&V) -> T2,
    G3: Fn(&V) -> T3,
    G4: Fn(&V) -> T4,
    F: Fn(T1, T2, T3, T4) -> V,
{
    Composite4Codec { codec1, getter1, codec2, getter2, codec3, getter3, codec4, getter4, constructor }
}
