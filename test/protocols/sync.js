'use strict';

const mocha = require('mocha');
const expect = require('chai').expect;
const stream_buf = require('stream-buffers');

const sync = require('joystream/protocols/sync');

function* id_generator(ids)
{
  for (var i = 0 ; i < ids.length ; ++i) {
    yield ids[i];
  }
  return 2;
}

function* error_id_generator()
{
  yield 'error';
  return 1;
}

function* missing_id_generator()
{
  yield 'missing';
  return 1;
}

function read_opener(id)
{
  if (id != 'foo' && id != 'bar' && id != 'error') {
    throw new Error(`Invalid ID "${id}"`);
  }

  const buf = new stream_buf.ReadableStreamBuffer();
  buf.emit('open');
  if (id == 'foo') {
    buf.put('foo-data');
  }
  else if (id == 'bar') {
    buf.put('bar-data');
  }
  else if (id == 'error') {
    buf.emit('error', new Error('test error in stream'));
  }

  buf.stop();
  return buf;
}

describe('protocols/sync', function()
{
  it('streams two streams in sequence', function(done)
  {
    const foo_result = new stream_buf.WritableStreamBuffer();
    const bar_result = new stream_buf.WritableStreamBuffer();

    const proto1 = new sync.SyncProtocol({
      generator: id_generator(['foo', 'bar']),
      read_open: read_opener,
    });
    const proto2 = new sync.SyncProtocol({
      write_open: (id) => {
        id = id.toString();
        if (id == 'foo') {
          return foo_result;
        }
        if (id == 'bar') {
          return bar_result;
        }
        throw new Error(`Invalid ID "${id}"`);
      }
    });

    const expected_sequence = [
      [sync.MSG_START_ID, 'foo'],
      [sync.MSG_DATA, 'foo-data'],
      [sync.MSG_END_ID, 'foo'],
      [sync.MSG_START_ID, 'bar'],
      [sync.MSG_DATA, 'bar-data'],
      [sync.MSG_END_ID, 'bar'],
      [sync.MSG_FINALIZE],
    ];
    var produced_expected_offset = 0;
    var consumed_expected_offset = 0;

    proto1.initiate((err, type, data) => {
      expect(err).to.be.null;

      // Check that what's produced matches expecations
      const expected = expected_sequence[produced_expected_offset];
      ++produced_expected_offset;

      expect(type).to.equal(expected[0]);
      if (data) {
        expect(data.toString()).to.equal(expected[1]);
      }

      // Consume
      proto2.consume(type, data, (err2, type2, data2) => {
        expect(foo_result.getContentsAsString()).to.equal('foo-data');
        expect(bar_result.getContentsAsString()).to.equal('bar-data');
        done();
      });
    });
  });

  it('streams bi-directionally', function(done)
  {
    const foo_result = new stream_buf.WritableStreamBuffer();
    const bar_result = new stream_buf.WritableStreamBuffer();
    const write_open = (id) => {
      id = id.toString();
      if (id == 'foo') {
        return foo_result;
      }
      if (id == 'bar') {
        return bar_result;
      }
      throw new Error(`Invalid ID "${id}"`);
    };

    const proto1 = new sync.SyncProtocol({
      generator: id_generator(['foo']),
      read_open: read_opener,
      write_open: write_open,
    });
    const proto2 = new sync.SyncProtocol({
      generator: id_generator(['bar']),
      read_open: read_opener,
      write_open: write_open,
    });

    proto1.initiate((err, type, data) => {
      expect(err).to.be.null;

      proto2.consume(type, data, (err, type, data) => {
        expect(err).to.be.null;

        if (typeof type !== 'undefined') {
          proto1.consume(type, data, (err, type, data) => {
            expect(err).to.be.null;

            if (typeof type === 'undefined') {
              // All done
              expect(foo_result.getContentsAsString()).to.equal('foo-data');
              expect(bar_result.getContentsAsString()).to.equal('bar-data');
              done();
            }
          });
        }
      });
    });
  });

  it('handles errors during opening', function(done)
  {
    const proto = new sync.SyncProtocol({
      generator: missing_id_generator(),
      read_open: read_opener,
    });

    proto.initiate((err, type, data) => {
      expect(err).not.to.be.null;
      done();
    });
  });

  it('handles errors during streaming', function(done)
  {
    const proto = new sync.SyncProtocol({
      generator: error_id_generator(),
      read_open: read_opener,
    });

    proto.initiate((err, type, data) => {
      expect(err).not.to.be.null;
      done();
    });

  });
});
