use super::*;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

/// Specifies a bijection between T and Wood
pub trait Biwooder<T>: Wooder<T> + Dewooder<T> {}

impl<T, X> Biwooder<T> for X where X: Wooder<T> + Dewooder<T> {}

/// Used by woods_derive, but you can use it too if you want. It's for scanning over structs with named pairs where the order is usually the same. Guesses that each queried field will be after the last, and only does a full scanaround when it finds that's not the case.
pub struct FieldScanning<'a> {
    pub v: &'a Wood,
    pub li: &'a [Wood],
    pub eye: usize,
}
impl<'a> FieldScanning<'a> {
    pub fn new(v: &'a Wood) -> Self {
        FieldScanning {
            v: v,
            li: v.tail().as_slice(),
            eye: 0,
        }
    }
    pub fn find(&mut self, key: &str) -> Result<&Wood, Box<WoodError>> {
        for _ in 0..self.li.len() {
            let c = &self.li[self.eye];
            if c.initial_str() == key {
                return if let Some(s) = c.tail().next() {
                    Ok(s)
                } else {
                    Err(Box::new(WoodError::new(
                        c,
                        format!("expected a subwood, but the wood has no tail"),
                    )))
                };
            }
            self.eye += 1;
            if self.eye >= self.li.len() {
                self.eye = 0;
            }
        }
        Err(Box::new(WoodError::new(
            self.v,
            format!("could not find key \"{}\"", key),
        )))
    }
}

/// A Biwooder that just uses the type's Woodable and Dewoodable impls
#[derive(Clone)]
pub struct Iden;
impl<T> Wooder<T> for Iden
where
    T: Woodable,
{
    fn woodify(&self, v: &T) -> Wood {
        v.woodify()
    }
}
impl<T> Dewooder<T> for Iden
where
    T: Dewoodable,
{
    fn dewoodify(&self, v: &Wood) -> Result<T, Box<WoodError>> {
        T::dewoodify(v)
    }
}

/// A biwooder that takes any valid expression of a bool, but produces the "yes"/"no" forms when wooding
#[derive(Clone)]
pub struct YesNo;
impl Wooder<bool> for YesNo {
    fn woodify(&self, v: &bool) -> Wood {
		if *v {
            "yes".into()
        } else {
            "no".into()
        }
    }
}
impl Dewooder<bool> for YesNo {
    fn dewoodify(&self, v: &Wood) -> Result<bool, Box<WoodError>> {
        bool::dewoodify(v)
    }
}

pub struct LambdaWooder<L>(L);
impl<W, L> Wooder<W> for LambdaWooder<L>
where
    L: Fn(&W) -> Wood,
{
    fn woodify(&self, v: &W) -> Wood {
        self.0(v)
    }
}
pub struct LambdaDewooder<D>(D);
impl<D, L> Dewooder<D> for LambdaDewooder<L>
where
    L: Fn(&Wood) -> Result<D, Box<WoodError>>,
{
    fn dewoodify(&self, v: &Wood) -> Result<D, Box<WoodError>> {
        self.0(v)
    }
}

pub struct CompositeBiwooder<W, D>(W, D);
impl<T, W, D> Wooder<T> for CompositeBiwooder<W, D>
where
    W: Wooder<T>,
{
    fn woodify(&self, v: &T) -> Wood {
        self.0.woodify(v)
    }
}
impl<T, W, D> Dewooder<T> for CompositeBiwooder<W, D>
where
    D: Dewooder<T>,
{
    fn dewoodify(&self, v: &Wood) -> Result<T, Box<WoodError>> {
        Dewooder::dewoodify(&self.1, v)
    }
}

/// you might want this for initializing biwooders that can't be constexprs
pub struct OptionalBoxBiwooder<B: ?Sized>(Option<Box<B>>);
impl<T, B> Wooder<T> for OptionalBoxBiwooder<B>
where
    B: Wooder<T> + ?Sized,
{
    fn woodify(&self, v: &T) -> Wood {
        self.0.as_ref().unwrap().woodify(v)
    }
}
impl<T, B> Dewooder<T> for OptionalBoxBiwooder<B>
where
    B: Dewooder<T> + ?Sized,
{
    fn dewoodify(&self, v: &Wood) -> Result<T, Box<WoodError>> {
        Dewooder::dewoodify(&**self.0.as_ref().unwrap(), v)
    }
}
impl<B> OptionalBoxBiwooder<B>
where
    B: ?Sized,
{
    pub const fn new(b: Box<B>) -> Self {
        OptionalBoxBiwooder(Some(b))
    }
    pub const fn empty() -> Self {
        OptionalBoxBiwooder(None)
    }
}

pub const fn biwooder_from_fns<W, D>(
    wf: W,
    df: D,
) -> CompositeBiwooder<LambdaWooder<W>, LambdaDewooder<D>> {
    CompositeBiwooder(LambdaWooder(wf), LambdaDewooder(df))
}

#[derive(Copy, Clone)]
pub struct SequenceBi<SubTran>(pub SubTran);
impl<T, SubTran> Wooder<Vec<T>> for SequenceBi<SubTran>
where
    SubTran: Wooder<T>,
{
    fn woodify(&self, v: &Vec<T>) -> Wood {
        let mut ret = Vec::new();
        woodify_seq_into(&self.0, v.iter(), &mut ret);
        ret.into()
    }
}
impl<T, SubTran> Dewooder<Vec<T>> for SequenceBi<SubTran>
where
    SubTran: Dewooder<T>,
{
    fn dewoodify(&self, v: &Wood) -> Result<Vec<T>, Box<WoodError>> {
        let mut ret = Vec::new();
        dewoodify_seq_into(&self.0, v.contents(), &mut ret)?;
        Ok(ret)
    }
}

#[derive(Copy, Clone)]
pub struct TaggedSequenceBi<'a, SubTran>(pub &'a str, pub SubTran);
impl<'a, T, SubTran> Wooder<Vec<T>> for TaggedSequenceBi<'a, SubTran>
where
    SubTran: Wooder<T>,
{
    fn woodify(&self, v: &Vec<T>) -> Wood {
        let mut ret = Vec::new();
        ret.push(self.0.into());
        woodify_seq_into(&self.1, v.iter(), &mut ret);
        ret.into()
    }
}

fn ensure_tag<'b>(v: &'b Wood, tag: &str) -> Result<std::slice::Iter<'b, Wood>, Box<WoodError>> {
    let mut i = v.contents();
    if let Some(name_wood) = i.next() {
        match *name_wood {
            Leafv(ref at) => {
                let name = at.v.as_str();
                if name == tag {
                    Ok(i)
                } else {
                    Err(Box::new(WoodError::new(
                        name_wood,
                        format!(
                            "expected \"{}\" here, but instead there was \"{}\"",
                            tag, name
                        ),
                    )))
                }
            }
            _ => Err(Box::new(WoodError::new(
                name_wood,
                format!(
                    "expected \"{}\" here, but instead there was a branch wood",
                    tag
                ),
            ))),
        }
    } else {
        Err(Box::new(WoodError::new(
            v,
            format!("expected \"{}\" at beginning, but the wood was empty", tag),
        )))
    }
}

impl<'a, T, SubTran> Dewooder<Vec<T>> for TaggedSequenceBi<'a, SubTran>
where
    SubTran: Dewooder<T>,
{
    fn dewoodify(&self, v: &Wood) -> Result<Vec<T>, Box<WoodError>> {
        let mut ret = Vec::new();
        let it = ensure_tag(v, &self.0)?;
        dewoodify_seq_into(&self.1, it, &mut ret)?;
        Ok(ret)
    }
}

fn dewoodify_pair<K, V, KeyTran, ValTran>(
    kt: &KeyTran,
    vt: &ValTran,
    v: &Wood,
) -> Result<(K, V), Box<WoodError>>
where
    KeyTran: Dewooder<K>,
    ValTran: Dewooder<V>,
{
    match *v {
        Branchv(ref lc) => {
            if lc.v.len() == 2 {
                unsafe {
                    let k = kt.dewoodify(lc.v.get_unchecked(0))?; // safe: we just checked the length
                    let v = vt.dewoodify(lc.v.get_unchecked(1))?; //
                    Ok((k, v))
                }
            } else {
                Err(Box::new(WoodError::new(
                    v,
                    format!(
                        "expected a pair, two elements, but the branch here has {}",
                        lc.v.len()
                    ),
                )))
            }
        }
        Leafv(_) => Err(Box::new(WoodError::new(
            v,
            "expected a pair, but the wood here is an leaf".into(),
        ))),
    }
}

#[derive(Copy, Clone)]
pub struct PairBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Wooder<(K, V)> for PairBi<KeyTran, ValTran>
where
    KeyTran: Wooder<K>,
    ValTran: Wooder<V>,
{
    fn woodify(&self, v: &(K, V)) -> Wood {
        let kt = self.0.woodify(&v.0);
        let vt = self.1.woodify(&v.1);
        branch!(kt, vt).into()
    }
}
impl<K, V, KeyTran, ValTran> Dewooder<(K, V)> for PairBi<KeyTran, ValTran>
where
    KeyTran: Dewooder<K>,
    ValTran: Dewooder<V>,
{
    fn dewoodify(&self, v: &Wood) -> Result<(K, V), Box<WoodError>> {
        dewoodify_pair(&self.0, &self.1, v)
    }
}

fn woodify_map<'a, K, V, KeyWooder, ValWooder, I>(
    ktr: &KeyWooder,
    vtr: &ValWooder,
    i: I,
    o: &mut Vec<Wood>,
) where
    KeyWooder: Wooder<K>,
    ValWooder: Wooder<V>,
    I: Iterator<Item = (&'a K, &'a V)>,
    K: 'a,
    V: 'a,
{
    for (kr, vr) in i {
        o.push(branch!(ktr.woodify(kr), vtr.woodify(vr)).into())
    }
}

fn dewoodify_map<'a, K, V, KeyTran, ValTran, I>(
    ktr: &KeyTran,
    vtr: &ValTran,
    i: I,
    o: &mut Vec<(K, V)>,
) -> Result<(), Box<WoodError>>
where
    KeyTran: Dewooder<K>,
    ValTran: Dewooder<V>,
    I: Iterator<Item = &'a Wood>,
{
    for v in i {
        o.push(dewoodify_pair(ktr, vtr, v)?);
    }
    Ok(())
}

impl<K, V> Woodable for HashMap<K, V>
where
    K: Eq + Hash + Woodable,
    V: Eq + Hash + Woodable,
{
    fn woodify(&self) -> Wood {
        let mut ret = Vec::new();
        woodify_map(&Iden, &Iden, self.iter(), &mut ret);
        ret.into()
    }
}
impl<K, V> Dewoodable for HashMap<K, V>
where
    K: Eq + Hash + Dewoodable,
    V: Eq + Hash + Dewoodable,
{
    fn dewoodify(v: &Wood) -> Result<HashMap<K, V>, Box<WoodError>> {
        let mut ret = Vec::new();
        dewoodify_map(&Iden, &Iden, v.contents(), &mut ret)?;
        Ok(HashMap::from_iter(ret.into_iter()))
    }
}

#[derive(Clone)]
pub struct HashMapBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Wooder<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
where
    KeyTran: Wooder<K>,
    ValTran: Wooder<V>,
    K: Eq + Hash,
    V: Eq + Hash,
{
    fn woodify(&self, v: &HashMap<K, V>) -> Wood {
        let mut ret = Vec::new();
        woodify_map(&self.0, &self.1, v.iter(), &mut ret);
        ret.into()
    }
}
impl<K, V, KeyTran, ValTran> Dewooder<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
where
    KeyTran: Dewooder<K>,
    ValTran: Dewooder<V>,
    K: Eq + Hash,
    V: Eq + Hash,
{
    fn dewoodify(&self, v: &Wood) -> Result<HashMap<K, V>, Box<WoodError>> {
        let mut ret = Vec::new();
        dewoodify_map(&self.0, &self.1, v.contents(), &mut ret)?;
        Ok(HashMap::from_iter(ret.into_iter()))
    }
}

#[derive(Clone)]
pub struct TaggedHashMapBi<'a, KeyTran, ValTran>(&'a str, KeyTran, ValTran);
impl<'a, K, V, KeyTran, ValTran> Wooder<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
where
    KeyTran: Wooder<K>,
    ValTran: Wooder<V>,
    K: Eq + Hash,
    V: Eq + Hash,
{
    fn woodify(&self, v: &HashMap<K, V>) -> Wood {
        let mut ret = Vec::new();
        ret.push(self.0.into());
        woodify_map(&self.1, &self.2, v.iter(), &mut ret);
        ret.into()
    }
}
impl<'a, K, V, KeyTran, ValTran> Dewooder<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
where
    KeyTran: Dewooder<K>,
    ValTran: Dewooder<V>,
    K: Eq + Hash,
    V: Eq + Hash,
{
    fn dewoodify(&self, v: &Wood) -> Result<HashMap<K, V>, Box<WoodError>> {
        let mut ret = Vec::new();
        let it = ensure_tag(v, self.0)?;
        dewoodify_map(&self.1, &self.2, it, &mut ret)?;
        Ok(HashMap::from_iter(ret.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent_int() {
        assert!(90isize == Iden.dewoodify(&Iden.woodify(&90isize)).unwrap());
    }

    #[test]
    fn tricky_branch_parse() {
        let brancho = branch!(branch!("tricky", "branch"), branch!("parse"));
        let tranner: SequenceBi<SequenceBi<Iden>> = SequenceBi(SequenceBi(Iden));
        let lv: Vec<Vec<String>> = tranner.dewoodify(&brancho).unwrap();
        assert!(lv.len() == 2);
        assert!(lv[0].len() == 2);
        assert!(lv[0][0].len() == 6);
        assert!(tranner.woodify(&lv).to_string() == "((tricky branch) (parse))");
    }

    #[test]
    fn do_hash_map() {
        let t = parse_termpose("a:b c:d d:e e:f").unwrap();
        let bt = TaggedHashMapBi("ob", Iden, Iden);
        let utr: Result<HashMap<String, String>, Box<WoodError>> = bt.dewoodify(&t);
        assert!(utr.is_err());
        let tt = parse_termpose("ob a:b c:d d:e e:f").unwrap();
        let hm: HashMap<String, String> = TaggedHashMapBi("ob", Iden, Iden).dewoodify(&tt).unwrap();
        assert!(hm.get("a").unwrap() == "b");
        assert!(hm.get("d").unwrap() == "e");
    }

    fn give_hm() -> HashMap<String, String> {
        [("a".into(), "b".into()), ("c".into(), "d".into())]
            .iter()
            .cloned()
            .collect()
    }

    #[test]
    fn implicit_biwooders() {
        let t = parse_termpose("a:b c:d").unwrap();
        let exh = give_hm();

        let exu: HashMap<String, String> = dewoodify(&t).unwrap();

        assert!(exu == exh)
    }

    const SPECIAL_SEQ_BIWOODER: SequenceBi<Iden> = SequenceBi(Iden);
    #[test]
    fn static_biwooder() {
        let r: Result<Wood, _> = parse_termpose("c c c c c a");
        let v: Vec<char> = SPECIAL_SEQ_BIWOODER.dewoodify(&r.unwrap()).unwrap();
        assert_eq!(v.len(), 6usize);
        assert_eq!(v[5], 'a');
    }

    #[test]
    fn automatic_deserialize() {
        let hm: HashMap<String, String> = deserialize("a:b c:d").unwrap();
        assert!(hm == give_hm());
    }

    #[test]
    fn automatic_serialize() {
        let hm = give_hm();
        let cln = deserialize(&serialize(&hm)).unwrap();
        assert!(hm == cln);
    }
	
	#[test]
	fn yesno() {
		assert_eq!("yes", YesNo.woodify(&true).initial_str());
	}
}
