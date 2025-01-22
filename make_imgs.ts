import { exec } from "https://deno.land/x/exec/mod.ts";

const elements = new Set(["air", "earth", "fire", "paper", "rock", "scissors", "water"]);
const augments = new Set(["armored", "combo", "parry"]);

for (const element of elements) {
  const makeBare = `
    magick -size 128x128 -depth 8 xc:none \
      ./assets/sources/${element}.png -composite \
      ./assets/${element}.png
  `;
  await exec(makeBare)

  for (const augment of augments) {
    const makeAugmentedEnchanted = `
      magick -size 128x128 -depth 8 xc:none \
        ./assets/sources/${element}.png                       -composite \
        ./assets/sources/${augment}.png  -geometry 48x48+8+72 -composite \
        ./assets/${element}-${augment}.png
    `;
    await exec(makeAugmentedEnchanted);
  }

  const aspects = new Set(elements);
  aspects.delete(element);
  for (const aspect of aspects) {
    const makeEnchanted = `
      magick -size 128x128 -depth 8 xc:none \
        ./assets/sources/${element}.png                       -composite \
        ./assets/sources/${aspect}.png  -geometry 48x48+72+72 -composite \
        ./assets/${element}-${aspect}.png
    `;
    await exec(makeEnchanted);

    for (const augment of augments) {
      const makeAugmentedEnchanted = `
        magick -size 128x128 -depth 8 xc:none \
          ./assets/sources/${element}.png                       -composite \
          ./assets/sources/${augment}.png  -geometry 48x48+8+72 -composite \
          ./assets/sources/${aspect}.png  -geometry 48x48+72+72 -composite \
          ./assets/${element}-${aspect}-${augment}.png
      `;
      await exec(makeAugmentedEnchanted);
    }
  }
}

