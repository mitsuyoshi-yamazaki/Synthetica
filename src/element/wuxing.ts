import { Element } from "./element"

export type WuxingAxis = "wood" | "fire" | "earth" | "metal" | "water"

export class Wuxing implements Element<WuxingAxis> {
  public constructor(
    public readonly wood: number,
    public readonly fire: number,
    public readonly earth: number,
    public readonly metal: number,
    public readonly water: number,
  ) {
  }

  public static zero(): Wuxing {
    return new Wuxing(0, 0, 0, 0, 0)
  }
}