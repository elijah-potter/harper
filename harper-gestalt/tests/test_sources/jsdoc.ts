/** This is a doc comment.
  * Since there are no keywords it _sould_ be checked. */
function test(){}

/** This is also a doc comment.
  * @class this sould be unchecked. */
class Clazz { }

/** Here is another example: {@link this sould also b unchecked}. But this _sould_ be.*/

/** However, tis should be checked, while {@link tis should not} */

/**
 * The following examples should be ignored by Harper.
 *
 * @param {string} n - ignor
 * @param {string} [o] - ignor
 * @param {string} [d=DefaultValue] - ignor
 * @return {string} ignor
 *
 * This should not be ignor
 */

function foo(n, o, d) {
  return n
}
