# another-mq

`another-mq` is another implementation of message-queue system that's I'm doing for fun during this summer.
It does not aim to production ready but to be an experiment with Rust asynchronous ecosystem.

## What is a message queue system?

A message queue system is a software collecting messages sent by sender applications which are stored into a queue
until the intended receiver dequeue them. This receiver can either be unspecified or specified.

It's a useful tool when you want to dispatch workloads from a web service, for instance, to avoid to wait the completion
of the task.

## Is this a clone of [RabbitMQ](https://rabbitmq.com)?

In a sense, yes. I intent to implement the [AMQP 0.9.1](https://www.rabbitmq.com/resources/specs/amqp0-9-1.pdf) specification
so in theory this software can be used as a replacement of RabbitMQ. However, it's not a rewrite of RabbitMQ, nor it's a
try to replace it. It's a summer experiment.

## License

For now, I'm not licensing this work. But maybe in the future.
