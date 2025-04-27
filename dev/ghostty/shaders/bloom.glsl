/**
 * Use the golden spiral pattern to sample around the current pixel.
 * Represented as (X, Y, Weight). Weight is the inverse of distance.
 */
const vec3[24] goldenSpiralSamples = {
	// ----------------------+---------------------+--------------------- // -------------------------- //
	//    X                    Y                    Weight                //  A Golden Spiral           //
	// ----------------------+---------------------+--------------------- // -------------------------- //
	vec3( 0.1693761725038636,  0.98555147617358950, 1.00000000000000000), //                            //
	vec3(-1.3330708309629430,  0.47214633286277730, 0.70710678118654750), //           ******           //
	vec3(-0.8464394909806497, -1.51113870578065000, 0.57735026918962580), //        ***     ***         //
	vec3( 1.5541556807284630, -1.25880900857097760, 0.50000000000000000), //       **         ***       //
	vec3( 1.6813643775894610,  1.47411459180526560, 0.44721359549995790), //      **            **      //
	vec3(-1.2795157692199817,  2.08874110322878400, 0.40824829046386310), //      *              *      //
	vec3(-2.4575847530631187, -0.97993733550247560, 0.37796447300922720), //     **               *     //
	vec3( 0.5874641440200847, -2.76674644293450770, 0.35355339059327373), //     *                **    //
	vec3( 2.9977157033697260,  0.11704939884745152, 0.33333333333333333), //     *     **          *    //
	vec3( 0.4136084245168839,  3.13511213055748030, 0.31622776601683794), //     **     *          *    //
	vec3(-3.1671499337692430,  0.98445990117702560, 0.30151134457776363), //      *     *           *   //
	vec3(-1.5736713846521535, -3.08602630791232450, 0.28867513459481290), //      **    *           *   //
	vec3( 2.8882026483404220, -2.15830615578962130, 0.27735009811261460), //       **  **           *   //
	vec3( 2.7150778983300325,  2.57455860411057150, 0.26726124191242440), //        ****            *   //
	vec3(-2.1504069972377464,  3.22114106276501650, 0.25819888974716110), //                        *   //
	vec3(-3.6548858794907493, -1.62536433081913430, 0.25000000000000000), //                        *   //
	vec3( 1.0130775986052671, -3.99670786763358340, 0.24253562503633297), //  Fibonacci's dance     *   //
	vec3( 4.2297236736072570,  0.33081361055181563, 0.23570226039551587), //  Golden ratio unfolds  *   //
	vec3( 0.4010779029117383,  4.34040741357259300, 0.22941573387056174), //  Yellow elegance      **   //
	vec3(-4.3191245702360280,  1.15981159969343800, 0.22360679774997896), //                       *    //
	vec3(-1.9209044802827355, -4.16054395213290700, 0.21821789023599240), //  - Nutty Versal       *    //
	vec3( 3.8639122286635708, -2.65898143829251230, 0.21320071635561041), //                      **    //
	vec3( 3.3486228404946234,  3.43318002326090000, 0.20851441405707477), //                     *      //
	vec3(-2.8769733643574344,  3.96522688641871570, 0.20412414523193154)  //                            //
	// ----------------------+---------------------+--------------------- // -------------------------- //
};

/**
 * Use the standard Rec. 601 luma formula, widely used in video and graphics.
 * It is perceptually accurate for human vision - we are more sensitive to
 * greens than blues.
 */
float calculateLuminance(vec4 color) {
	return 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
}

/**
 * Calculates a time-based glimmering effect that pulses at a regular interval.
 * It creates a smooth, oscillating intensity value that varies over time,
 * producing a subtle pulsing glow effect.
 */
float calculateTemporalGlimmer() {
	float intensity = 0.3;
	float amplitude = 0.3;
	float shift = 0.5;
	float speed = 3.0;
	float glimmer = intensity * (amplitude * sin(iTime * speed) + shift);

	return glimmer;
}

/**
 * Calculates a position-based glimmering effect that creates diagonal patterns.
 * It generates a subtle glimmer pattern based on the location of the current
 * pixel, which moves diagonally across the screen over time, creating spatial
 * variation in the bloom effect.
 */
float calculateSpatialGlimmer(vec2 uv) {
	float intensity = 0.069;
	float frequency = 24.0;
	float speed = 0.4;
	float translation = iTime * speed;
	float diagonal = translation + (uv.x + uv.y) / 2.0;
	float glimmer = intensity * sin(diagonal * frequency);

	return glimmer;
}

/**
 * Creates a dynamic bloom effect with spatial & temporal glimmering.
 * It creates a soft glow around bright objects and makes them look shiny.
 */
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
	vec2 uv = fragCoord.xy / iResolution.xy;
	vec4 color = texture(iChannel0, uv);

	for (int i = 0; i < 24; i++) {
		// Take a golden spiral sample.
		vec2 step = vec2(1.414) / iResolution.xy;
		vec2 offset = goldenSpiralSamples[i].xy;
		float weight = goldenSpiralSamples[i].z;
		vec4 light = texture(iChannel0, uv + offset * step);

		// Measure the luminance of the sample.
		float luminance = calculateLuminance(light);
		float brightnessThreshold = 0.420;

		// Add some glimmer! ✨
		float temporalGlimmer = calculateTemporalGlimmer();
		float spatialGlimmer = calculateSpatialGlimmer(uv);
		float glimmer = mix(temporalGlimmer, spatialGlimmer, 0.69);

		// Illuminate the bright pixels! ⚡️
		if (luminance > brightnessThreshold) {
			color += light * luminance * weight * glimmer;
		}
	}

	fragColor = color;
}
