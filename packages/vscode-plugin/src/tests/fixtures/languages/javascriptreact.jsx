/**
 * @param {{message: string}} props Errorz
 */
export default function Main(props = { message: 'Hello World!' }) {
	return <h1>{props.message}</h1>;
}
