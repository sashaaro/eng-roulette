import { EventEmitter } from 'eventemitter3'

export type Callback<A> = (a: A) => void

// eslint-disable-next-line
export type Events = Record<string | symbol, any>

export type TypedEmitterKeys =
    'removeListener' |
    'on' |
    'once' |
    'off' |
    'emit'

// Some methods might be missing and we do not extend EventEmitter because
// SocketIOClient.Socket does not inherit from EventEmitter, and the method
// signatures differ slightly.
export interface TypedEmitter<E extends Events> {
    removeListener<K extends keyof E>(t: K, callback: Callback<E[K]>): this
    removeAllListeners(event?: string): void

    on<K extends keyof E>(t: K, callback: Callback<E[K]>): this
    once<K extends keyof E>(t: K, callback: Callback<E[K]>): this

    emit<K extends keyof E>(t: K, value: E[K]): void
}


export class SimpleEmitter<E extends Events>
    implements TypedEmitter<E> {
    protected readonly emitter = new EventEmitter()

    removeAllListeners(event?: string) {
        if (arguments.length === 0) {
            this.emitter.removeAllListeners()
        } else {
            this.emitter.removeAllListeners(event)
        }
        // this.ws.removeEventListener('close', this.wsHandleClose)
        // this.ws.removeEventListener('open', this.wsHandleOpen)
        // this.ws.removeEventListener('message', this.wsHandleMessage)
    }

    removeListener<K extends keyof E>(name: K, callback: Callback<E[K]>): this {
        this.emitter.removeListener(name as string, callback)
        return this
    }

    on<K extends keyof E>(name: K, callback: Callback<E[K]>): this {
        this.emitter.on(name as string, callback)
        return this
    }

    once<K extends keyof E>(name: K, callback: Callback<E[K]>): this {
        this.emitter.once(name as string, callback)
        return this
    }

    emit<K extends keyof E>(name: K, value: E[K]): void {
        this.emitter.emit(name as string, value)
    }
}