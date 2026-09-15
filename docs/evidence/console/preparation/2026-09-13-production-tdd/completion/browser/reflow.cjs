const {expect}=require('@playwright/test');
// Scalar composer acceptance only; grid scrolling has a separate contract.
async function expectEssentialReflow(page,controls) {
  const metrics=await page.evaluate(()=>({width:document.documentElement.clientWidth,scroll:Math.max(document.documentElement.scrollWidth,document.body.scrollWidth)}));
  expect(metrics.scroll,'page must not require horizontal scrolling').toBeLessThanOrEqual(metrics.width+1);
  for(const control of controls) {
    await expect(control).toBeVisible();
    const before=await control.boundingBox();
    expect(before,'essential control needs a rendered box').not.toBeNull();
    expect(before.x,'essential control starts within horizontal viewport').toBeGreaterThanOrEqual(0);
    expect(before.x+before.width,'essential control ends within horizontal viewport').toBeLessThanOrEqual(metrics.width+1);
    // Vertical scrolling is permitted. Initial horizontal bounds are checked
    // before any automatic scrolling can hide an offscreen-control defect.
    await control.scrollIntoViewIfNeeded();
    await expect(control).toBeInViewport({ratio:1});
  }
}
module.exports={expectEssentialReflow};
