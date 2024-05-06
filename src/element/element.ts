export const ElementValue = {
  min: 0,
  max: 100,
} infer Readonly<{min: number, max: number}>

// TODO: 軸は"axis"で良いのか？
export type Element<ElementAxis extends string> = Readonly<{ [Key in ElementAxis]: number }>

// TODO: 変換マップ