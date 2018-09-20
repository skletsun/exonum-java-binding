package com.exonum.binding.fakes.test;

import com.exonum.binding.transaction.TransactionExecutionException;

import javax.annotation.Nullable;

/**
 * Used in tests that cover the cases of using subclass of {@link #TransactionExecutionException}.
 */
public class TestTxExecException extends TransactionExecutionException {

  /**
   * The constructor that gets called from native code.
   *
   * @param errorCode the transaction error code
   * @param description the error description. The detail description is saved for
   *                    later retrieval by the {@link #getMessage()} method.
   */
  public TestTxExecException(byte errorCode, @Nullable String description) {
    super(errorCode, description);
  }
}
