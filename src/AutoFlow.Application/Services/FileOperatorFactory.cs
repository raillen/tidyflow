using System;
using System.Collections.Generic;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.Application.Services;

public class FileOperatorFactory
{
    private readonly IEnumerable<IFileOperator> _operators;

    public FileOperatorFactory(IEnumerable<IFileOperator> operators)
    {
        _operators = operators;
    }

    public IFileOperator GetOperator(string sourcePath, string targetPath)
    {
        // Simple implementation: if either path starts with sftp://, find the SftpFileOperator.
        // In a real system, you might inspect the job configuration for credentials.
        if (sourcePath.StartsWith("sftp://") || targetPath.StartsWith("sftp://"))
        {
            foreach (var op in _operators)
            {
                if (op.GetType().Name == "SftpFileOperator")
                {
                    return op;
                }
            }
        }
        
        // Default to LocalFileOperator
        foreach (var op in _operators)
        {
            if (op.GetType().Name == "LocalFileOperator")
            {
                return op;
            }
        }

        throw new InvalidOperationException("No suitable file operator found.");
    }
}
